use super::types;

pub trait Provide {
    fn notes(&self) -> &[types::Note];
}
