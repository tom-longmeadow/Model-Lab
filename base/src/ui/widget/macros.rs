
 

macro_rules! impl_widget_base {
    ($t:ty) => {
        impl $t {
            pub fn id(&self) -> $crate::ui::widget::WidgetId {
                self.base.id()
            }

            pub fn rect(&self) -> $crate::ui::widget::layout::rect::Rect {
                self.base.box_model().rect()
            }

            pub fn set_rect(&mut self, rect: $crate::ui::widget::layout::rect::Rect) {
                let mut model = self.base.box_model();
                model.set_rect(rect);
                self.base.set_box_model(model);
            }

            pub fn with_rect(
                mut self,
                rect: $crate::ui::widget::layout::rect::Rect,
            ) -> Self {
                self.set_rect(rect);
                self
            }

            pub fn padding(&self) -> $crate::ui::widget::layout::edge_insets::EdgeInsets {
                self.base.box_model().padding()
            }

            pub fn set_padding(
                &mut self,
                padding: $crate::ui::widget::layout::edge_insets::EdgeInsets,
            ) {
                let mut model = self.base.box_model();
                model.set_padding(padding);
                self.base.set_box_model(model);
            }

            pub fn with_padding(
                mut self,
                padding: $crate::ui::widget::layout::edge_insets::EdgeInsets,
            ) -> Self {
                self.set_padding(padding);
                self
            }

            pub fn background(&self) -> [u8; 4] {
                self.base.box_model().background()
            }

            pub fn set_background(&mut self, color: [u8; 4]) {
                let mut model = self.base.box_model();
                model.set_background(color);
                self.base.set_box_model(model);
            }

            pub fn with_background(mut self, color: [u8; 4]) -> Self {
                self.set_background(color);
                self
            }

            pub fn border(&self) -> $crate::ui::widget::layout::border::BorderStyle {
                self.base.box_model().border()
            }

            pub fn set_border(
                &mut self,
                border: $crate::ui::widget::layout::border::BorderStyle,
            ) {
                let mut model = self.base.box_model();
                model.set_border(border);
                self.base.set_box_model(model);
            }

            pub fn with_border(
                mut self,
                border: $crate::ui::widget::layout::border::BorderStyle,
            ) -> Self {
                self.set_border(border);
                self
            }
        }
    };
}

macro_rules! impl_widget_text {
    ($t:ty) => {
        impl $t {
            pub fn text(&self) -> &str {
                self.text.text()
            }

            pub fn with_text_style(
                mut self,
                style: $crate::ui::text::style::TextStyle,
            ) -> Self {
                self.text.set_style(style);
                self
            }

            pub fn set_text(&mut self, text: impl Into<String>) {
                self.text.set_text(text);
            }

            pub fn text_style(&self) -> $crate::ui::text::style::TextStyle {
                self.text.style()
            }

            pub fn set_text_style(&mut self, style: $crate::ui::text::style::TextStyle) {
                self.text.set_style(style);
            }
        }
    };
}

macro_rules! impl_widget_container {
    ($t:ty) => {
        impl $t {
            pub fn push(&mut self, child: Box<dyn $crate::ui::widget::Widget>) {
                self.container.push(child);
            }

            pub fn remove(&mut self, index: usize) -> Option<Box<dyn $crate::ui::widget::Widget>> {
                self.container.remove(index)
            }

            pub fn clear(&mut self) {
                self.container.clear();
            }

            pub fn children(&self) -> &[Box<dyn $crate::ui::widget::Widget>] {
                self.container.children()
            }

            pub fn gap(&self) -> f32 {
                self.container.gap()
            }

            pub fn set_gap(&mut self, gap: f32) {
                self.container.set_gap(gap);
            }

            pub fn with_gap(mut self, gap: f32) -> Self {
                self.container.set_gap(gap);
                self
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

        impl $crate::ui::widget::container::Container for $t {
            fn children(&self) -> &[Box<dyn Widget>] {
                self.container.children()
            }

            fn push(&mut self, child: Box<dyn Widget>) {
                self.container.push(child);
            }

            fn remove(&mut self, index: usize) -> Option<Box<dyn Widget>> {
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