use std::{collections::HashMap, sync::Arc};

use events::{EventResponse, EventTypes};
use nalgebra::Point2;
use render::{GpuBound, GpuProxy, RenderElement};
use styles::{Size, StyleSheet};

pub mod events;
pub mod nodes;
pub mod render;
pub mod styles;
pub mod texture;

pub struct Gui<Msg>
where
    Msg: Clone,
{
    nodes: HashMap<ElementKey, Element<Msg>>,
    entry: Option<ElementKey>,
    last_key: u64,
    size: (u32, u32),
    gpu: GpuBound,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ElementKey {
    id: u64,
}

impl<Msg> Gui<Msg>
where
    Msg: Clone,
{
    pub fn new(size: (u32, u32), device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let gpu = GpuBound::new(queue, device, size);
        let this = Self {
            nodes: HashMap::new(),
            last_key: 0,
            entry: None,
            size,
            gpu,
        };
        this
    }

    pub fn add_element(&mut self, element: Element<Msg>) -> ElementKey {
        let key = ElementKey { id: self.last_key };
        self.last_key += 1;
        self.nodes.insert(key, element);
        key
    }

    pub fn remove_node(&mut self, key: ElementKey) {
        self.nodes.remove(&key);
    }

    pub fn get_node(&self, key: ElementKey) -> Option<&Element<Msg>> {
        self.nodes.get(&key).map(|node| node)
    }

    pub fn get_node_mut(&mut self, key: ElementKey) -> Option<&mut Element<Msg>> {
        self.nodes.get_mut(&key).map(|node| node)
    }

    pub fn set_entry(&mut self, key: Option<ElementKey>) {
        if let Some(entry) = self.entry.take() {
            self.remove_node(entry);
        }
        self.entry = key;
        self.transform_entry(self.size);
    }

    pub fn event(&mut self, event: events::Event) -> EventResponse<Msg> {
        let entry_key = match &self.entry {
            Some(entry) => entry,
            None => return EventResponse::Ignored,
        };
        let node = match self.nodes.get(entry_key) {
            Some(node) => node,
            None => return EventResponse::Ignored,
        };
        todo!("Handle the event")
    }

    pub fn resize(&mut self, size: (u32, u32), queue: &wgpu::Queue) {
        self.size = size;
        self.gpu.resize((size.0, size.1), queue);
        let entry_key = if let Some(entry) = &self.entry {
            entry
        } else {
            return;
        };
        self.transform_entry(size);
    }

    fn transform_entry(&mut self, size: (u32, u32)) {
        let entry_key = match &self.entry {
            Some(entry) => entry,
            None => return,
        };
        self.transform_element(*entry_key, NodeTransform {
            position: Point2::new(size.0 as f32 / 2.0, size.1 as f32 / 2.0),
            scale: Point2::new(size.0 as f32, size.1 as f32),
            rotation: 0.0,
        });
    }

    fn transform_element(&mut self, key: ElementKey, transform: NodeTransform) {
        let node = match self.nodes.get_mut(&key) {
            Some(node) => node,
            None => return,
        };
        let styles = &node.styles;
        let (width, height) = (styles.get_width(transform.scale.x), styles.get_height(transform.scale.y));
        let transform = NodeTransform {
            position: Point2::new(transform.position.x, transform.position.y),
            scale: Point2::new(width, height),
            rotation: 0.0,
        };
        let color = styles.background.color;
        match &styles.background.texture {
            Some(texture) => {
                node.render_element.set_texture(texture.clone());
            }
            _ => {}
        }
        node.render_element.set_color(color, &self.gpu.proxy);
        node.render_element.set_transform(&transform, &self.gpu.proxy);
        match node.children.to_owned() {
            Children::Element(child) => {
                self.transform_element(child.clone(), transform);
                return;
            }
            Children::Layers(children) => todo!("Transform the children"),
            Children::Rows { children, .. } => todo!("Transform the children"),
            Children::Columns { children, .. } => todo!("Transform the children"),
            Children::None => return,
        };
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let entry_key = match &self.entry {
            Some(entry) => entry,
            None => return,
        };
        pass.set_bind_group(0, &self.gpu.dimensions_bind_group, &[]);
        let node = match self.nodes.get(entry_key) {
            Some(node) => node,
            None => return,
        };
        
        node.render_element.render(&self.gpu.proxy.pipelines, pass);
    }

    pub fn texture_from_bytes(&self, bytes: &[u8], label: &str) -> texture::Texture {
        texture::Texture::from_bytes(&self.gpu.proxy, bytes, label)
    }

    pub fn texture_from_image(&self, img: &image::DynamicImage, label: Option<&str>) -> texture::Texture {
        texture::Texture::from_image(&self.gpu.proxy, img, label)
    }

    pub fn tetxture_from_file(&self, path: &str) -> texture::Texture {
        let image = image::open(path).unwrap();
        self.texture_from_image(&image, None)
    }
}

#[derive(Clone, Debug)]
/// Transformation of a node
/// 
/// Node transformations are applied to the node and its children
/// when the node is rendered for the first time or when the node
/// or its parent is resized
pub struct NodeTransform {
    /// Position in x and y of the top left corner
    pub position: Point2<f32>,
    /// Scale in width and height
    pub scale: Point2<f32>,
    /// Rotation in radians
    pub rotation: f32,
}

pub struct Element <Msg> where Msg: Clone {
    pub label: Option<String>,
    pub render_element: RenderElement,
    pub styles: StyleSheet,
    pub event_listeners: HashMap<EventTypes, Msg>,
    pub children: Children,
}

impl <Msg> Element <Msg> where Msg: Clone {
    pub fn new(gui: &Gui<Msg>) -> Self {
        Self {
            label: None,
            render_element: RenderElement::zeroed(&gui.gpu.proxy),
            styles: StyleSheet::default(),
            event_listeners: HashMap::new(),
            children: Children::None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_styles(mut self, styles: StyleSheet) -> Self {
        self.styles = styles;
        self
    }

    pub fn with_event_listener(mut self, event: EventTypes, msg: Msg) -> Self {
        self.event_listeners.insert(event, msg);
        self
    }

    pub fn with_children(mut self, children: Children) -> Self {
        self.children = children;
        self
    }
}

#[derive(Clone, Debug)]
pub enum Children {
    Element(ElementKey),
    Layers(Vec<ElementKey>),
    Rows {
        children: Vec<ElementKey>,
        spacing: Size,
    },
    Columns {
        children: Vec<ElementKey>,
        spacing: Size,
    },

    None,
}