mod container;
mod note_tree;
mod id;

pub mod error;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn new(reader: R) -> Self {
        let container = container::Container::open(reader).expect("Failed to open container");
        Self { container }
    }

    pub fn container_type(&self) -> &container::ContainerType {
        self.container.container_type()
    }
}
