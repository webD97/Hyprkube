use rhai::{CustomType, TypeBuilder};
use serde::Serialize;

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredString {
    pub string: String,
    pub color: String,
}

impl ColoredString {
    pub fn new(string: String, color: String) -> Self {
        ColoredString { string, color }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ColoredString", Self::new);
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBox {
    pub color: String,
}

impl ColoredBox {
    pub fn new(string: String) -> Self {
        ColoredBox { color: string }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ColoredBox", Self::new);
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct Hyperlink {
    pub url: String,
    pub display_text: String,
}

impl Hyperlink {
    pub fn new(url: String, display_text: String) -> Self {
        Self { url, display_text }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("Hyperlink", Self::new);
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct RelativeTime {
    pub iso: String,
}

impl RelativeTime {
    pub fn new(iso: String) -> Self {
        Self { iso }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("RelativeTime", Self::new);
    }
}
