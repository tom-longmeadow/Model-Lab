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
        }
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
        }
    };
}

macro_rules! impl_widget_container {
    ($t:ty) => {
        impl $t {
            pub fn children(&self) -> &[Box<dyn $crate::ui::widget::Widget>] {
                self.container.children()
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