use anyhow::{Context, Result};
use comrak::nodes::NodeValue;
use comrak::{Arena, Options, format_html, parse_document};
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;

use crate::types;

/// Parses raw markdown sources into public notes.
///
/// Notes that fail parsing or are marked as private are skipped while logging
/// errors, preserving the current behavior.
pub fn notes(raw: Vec<types::note::Source>) -> Vec<types::note::Note> {
    raw.into_iter()
        .filter_map(|(path_buf, raw_md)| {
            let post_note_entry = match PostNoteEntry::new(&path_buf, &raw_md) {
                Ok(post_note_entry) => post_note_entry,
                Err(err) => {
                    log::error!(
                        "Something went wrong while parsing post note {:?}: {}",
                        &path_buf,
                        err
                    );
                    return None;
                }
            };

            let post_note = match post_note_entry {
                PostNoteEntry::Public(post_note) => post_note,
                PostNoteEntry::Private => {
                    log::info!("Skipping private note: {:?}", &path_buf);
                    return None;
                }
            };

            log::info!("Loaded public note: {:?}", &path_buf);

            Some(*post_note)
        })
        .collect()
}

/// Builds the content search map from parsed notes.
pub fn content(notes: &[types::note::Note]) -> types::map::Content<'_> {
    types::map::Content::from(notes)
}

/// Builds the navigation tree from parsed notes.
pub fn navigation(notes: &[types::note::Note]) -> types::map::Navigation {
    types::map::Navigation::from(notes)
}

enum PostNoteEntry {
    Public(Box<types::note::Note>),
    Private,
}

impl PostNoteEntry {
    fn new(file_name: &Path, raw_md: &str) -> Result<PostNoteEntry> {
        let (md, media) = match media(raw_md) {
            Ok((md, media)) => (md, media),
            Err(err) => {
                log::warn!("Could not pre-process media wikilinks: {}", err);
                (Cow::from(raw_md), Vec::new())
            }
        };

        let arena = Arena::new();
        let mut options = Options::default();

        options.extension.table = true;
        options.extension.math_dollars = true;
        options.extension.wikilinks_title_after_pipe = true;
        options.extension.front_matter_delimiter = Some("---".to_owned());

        let root = parse_document(&arena, &md, &options);

        let file_name = types::html::link::Internal::try_from(file_name.to_path_buf())?;
        let mut maybe_properties: Option<types::note::Frontmatter> = Option::None;
        let mut links: Vec<types::html::link::Internal> = Vec::new();

        for node in root.descendants() {
            match &mut node.data.borrow_mut().value {
                NodeValue::FrontMatter(raw_front_matter) => {
                    let raw_yml = raw_front_matter.replace("---", "").replace("\\n", "");
                    let front_matter: types::note::Frontmatter = serde_yaml::from_str(&raw_yml)?;

                    if !front_matter.public {
                        return Ok(Self::Private);
                    }

                    maybe_properties = Some(front_matter);
                }

                NodeValue::WikiLink(link) => {
                    let internal_link = types::html::link::Internal::from(link.url.to_owned());
                    link.url = internal_link.to_string();
                    links.push(internal_link);
                }

                // Clip everything that comes after `## Questions`. This is done because I'm to
                // busy to think of a propper way to render my anki cards.
                NodeValue::Heading(heading) => {
                    if heading.level == 2
                        && let Some(first_child) = node.first_child()
                    {
                        let borrowed = first_child.data.borrow();
                        if let NodeValue::Text(ref text) = borrowed.value
                            && text == "Questions"
                        {
                            let mut next_sibling = node.next_sibling();

                            while let Some(sibling) = next_sibling {
                                next_sibling = sibling.next_sibling();
                                sibling.detach();
                            }

                            if let Some(previous_sibling) = node.previous_sibling() {
                                previous_sibling.detach();
                            }

                            node.detach();

                            break;
                        }
                    }
                }

                _ => {}
            }
        }

        let properties = maybe_properties.context("Could not determine properties!")?;

        let mut html_buf = Vec::new();
        format_html(root, &options, &mut html_buf)?;

        let html = types::html::Html::try_from(html_buf)?;

        Ok(Self::Public(Box::new(types::note::Note::new(
            file_name, properties, links, media, html,
        ))))
    }
}

// This is probably going to be a temporary solution.
fn media(raw: &str) -> Result<(Cow<'_, str>, Vec<types::html::link::Media>)> {
    let re = Regex::new(r"!\[\[(media/[^|\]]+)(?:\|([^\[\]]+))?\]\]")?;
    let mut media_links = Vec::new();

    let md = re.replace_all(raw, |caps: &regex::Captures| {
        let link = types::html::link::Media::from(caps[1].to_string());
        let title = caps.get(2).map_or("", |m| m.as_str());

        media_links.push(link.clone());

        format!("![{}](./{})", title, link.to_string().replace(" ", "%20"))
    });

    Ok((md, media_links))
}
