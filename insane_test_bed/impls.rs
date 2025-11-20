pub struct NewsArticle { pub headline: String }
pub trait Name { fn name(&self) -> String; }
impl Name for NewsArticle { fn name(&self) -> String { self.headline.clone() } }