macro_rules! impl_widget_base {
    ($t:ty) => {
        impl $t {
            pub fn id(&self) -> $crate::ui::widget::WidgetId {
                self.base.id()
            }

            pub fn is_visible(&self) -> bool {
                self.base.is_visible()
            }

            pub fn set_visible(&mut self, visible: bool) {
                self.base.set_visible(visible);
            }

            pub fn hide(&mut self) {
                self.base.hide();
            }

            pub fn show(&mut self) {
                self.base.show();
            }

            pub fn with_hidden(mut self) -> Self {
                self.base.hide();
                self
            }

            pub fn rect(&self) -> $crate::ui::layout::rect::Rect {
                self.base.rect()
            }

            pub fn set_rect(&mut self, rect: $crate::ui::layout::rect::Rect) {
                self.base.set_rect(rect);
            }

            pub fn with_rect(mut self, rect: $crate::ui::layout::rect::Rect) -> Self {
                self.base.set_rect(rect);
                self
            }

            pub fn background(&self) -> Option<$crate::ui::layout::color::Color> {
                self.base.background()
            }

            pub fn set_background(&mut self, color: $crate::ui::layout::color::Color) {
                self.base.set_background(color);
            }

            pub fn clear_background(&mut self) {
                self.base.clear_background();
            }

            pub fn with_background(mut self, color: $crate::ui::layout::color::Color) -> Self {
                self.base.set_background(color);
                self
            }

            pub fn border(&self) -> Option<$crate::ui::layout::border::BorderStyle> {
                self.base.border()
            }

            pub fn set_border(&mut self, border: $crate::ui::layout::border::BorderStyle) {
                self.base.set_border(border);
            }

            pub fn clear_border(&mut self) {
                self.base.clear_border();
            }

            pub fn with_border(mut self, border: $crate::ui::layout::border::BorderStyle) -> Self {
                self.base.set_border(border);
                self
            }
        }

        // impl $crate::ui::widget::Widget for $t {
        //     fn base(&self) -> &$crate::ui::widget::WidgetBase {
        //         &self.base
        //     }
        // }
    };
}

macro_rules! impl_widget_text {
    ($t:ty) => {
        impl $t {
            pub fn text(&self) -> &str {
                self.text.text()
            }

            pub fn set_text(&mut self, text: impl Into<String>) {
                self.text.set_text(text);
            }

            pub fn style(&self) -> Option<$crate::ui::text::style::TextStyle> {
                self.text.style()
            }

            pub fn set_style(&mut self, style: $crate::ui::text::style::TextStyle) {
                self.text.set_style(style);
            }

            pub fn clear_style(&mut self) {
                self.text.clear_style();
            }

            pub fn with_style(mut self, style: $crate::ui::text::style::TextStyle) -> Self {
                self.text.set_style(style);
                self
            }

            pub fn resolved_style(
                &self,
                fallback: $crate::ui::text::style::TextStyle,
            ) -> $crate::ui::text::style::TextStyle {
                self.text.resolved_style(fallback)
            }
        }
    };
}

macro_rules! impl_widget_container {
    ($t:ty) => {
        impl $t {
            pub fn children(&self) -> &[Box<dyn $crate::ui::widget::Widget>] {
                self.container.children()
            }

            pub fn gap(&self) -> Option<f32> {
                self.container.gap()
            }

            pub fn set_gap(&mut self, gap: f32) {
                self.container.set_gap(gap);
            }

            pub fn clear_gap(&mut self) {
                self.container.clear_gap();
            }

            pub fn with_gap(mut self, gap: f32) -> Self {
                self.container.set_gap(gap);
                self
            }

            pub fn push(&mut self, child: Box<dyn $crate::ui::widget::Widget>) {
                self.container.push(child);
            }

            pub fn remove(&mut self, index: usize) -> Option<Box<dyn $crate::ui::widget::Widget>> {
                self.container.remove(index)
            }

            pub fn clear(&mut self) {
                self.container.clear();
            }

            pub fn with_child(mut self, child: Box<dyn $crate::ui::widget::Widget>) -> Self {
                self.container.push(child);
                self
            }

            pub fn with_children<I>(mut self, children: I) -> Self
            where
                I: IntoIterator<Item = Box<dyn $crate::ui::widget::Widget>>,
            {
                self.container.push_children(children);
                self
            }
        }

        impl $crate::ui::container::Container for $t {
            fn children(&self) -> &[Box<dyn $crate::ui::widget::Widget>] {
                self.container.children()
            }

            fn push(&mut self, child: Box<dyn $crate::ui::widget::Widget>) {
                self.container.push(child);
            }

            fn remove(&mut self, index: usize) -> Option<Box<dyn $crate::ui::widget::Widget>> {
                self.container.remove(index)
            }

            fn clear(&mut self) {
                self.container.clear();
            }
        }
    };
}
  
pub(crate) use impl_widget_base;
pub(crate) use impl_widget_text;
pub(crate) use impl_widget_container;