#[derive(Default, PartialEq, Eq)]
pub enum Category {
    #[default]
    Identity,
    Compute,
}

impl Category {
    pub fn from_type(type_: &str) -> Self {
        match type_ {
            "identity" => Category::Identity,
            "compute" => Category::Compute,
            _ => Category::Identity,
        }
    }
}
