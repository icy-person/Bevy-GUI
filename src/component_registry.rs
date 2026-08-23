use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentKind {
    Transform,
    Name,
    Visibility,
    Parent,
    Camera,
    Light,
    Mesh,
    Material,
    EditorEntity,
}

impl ComponentKind {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Transform => "Transform",
            Self::Name => "Name",
            Self::Visibility => "Visibility",
            Self::Parent => "Parent",
            Self::Camera => "Camera",
            Self::Light => "Light",
            Self::Mesh => "Mesh",
            Self::Material => "Material",
            Self::EditorEntity => "Editor Entity",
        }
    }

    pub fn category(self) -> &'static str {
        match self {
            Self::Transform => "Core",
            Self::Name => "Core",
            Self::Visibility => "Core",
            Self::Parent => "Core",
            Self::Camera => "Rendering",
            Self::Light => "Rendering",
            Self::Mesh => "Rendering",
            Self::Material => "Rendering",
            Self::EditorEntity => "Editor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyKind {
    Text,
    Bool,
    Float,
    Vec3,
    Color,
    Enum,
    ReadOnly,
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: &'static str,
    pub label: &'static str,
    pub kind: PropertyKind,
    pub editable: bool,
    pub tooltip: &'static str,
}

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    pub kind: ComponentKind,
    pub label: &'static str,
    pub category: &'static str,
    pub properties: &'static [PropertyDescriptor],
    pub removable: bool,
    pub addable: bool,
}

const TRANSFORM_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "translation", label: "Position", kind: PropertyKind::Vec3, editable: true, tooltip: "Local translation in world or parent space." },
    PropertyDescriptor { name: "rotation", label: "Rotation", kind: PropertyKind::Vec3, editable: true, tooltip: "Euler rotation in degrees." },
    PropertyDescriptor { name: "scale", label: "Scale", kind: PropertyKind::Vec3, editable: true, tooltip: "Local scale." },
];

const NAME_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "value", label: "Name", kind: PropertyKind::Text, editable: true, tooltip: "Display name shown in the hierarchy." },
];

const VISIBILITY_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "visible", label: "Visible", kind: PropertyKind::Bool, editable: true, tooltip: "Controls editor and runtime visibility." },
];

const PARENT_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "parent", label: "Parent", kind: PropertyKind::ReadOnly, editable: false, tooltip: "Owning hierarchy parent." },
];

const CAMERA_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "projection", label: "Projection", kind: PropertyKind::Enum, editable: true, tooltip: "Perspective or orthographic projection." },
    PropertyDescriptor { name: "near", label: "Near", kind: PropertyKind::Float, editable: true, tooltip: "Near clipping plane." },
    PropertyDescriptor { name: "far", label: "Far", kind: PropertyKind::Float, editable: true, tooltip: "Far clipping plane." },
];

const LIGHT_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor { name: "color", label: "Color", kind: PropertyKind::Color, editable: true, tooltip: "Light color." },
    PropertyDescriptor { name: "intensity", label: "Intensity", kind: PropertyKind::Float, editable: true, tooltip: "Light intensity." },
];

const EMPTY: &[PropertyDescriptor] = &[];

const DESCRIPTORS: &[ComponentDescriptor] = &[
    ComponentDescriptor { kind: ComponentKind::Transform, label: "Transform", category: "Core", properties: TRANSFORM_PROPERTIES, removable: false, addable: true },
    ComponentDescriptor { kind: ComponentKind::Name, label: "Name", category: "Core", properties: NAME_PROPERTIES, removable: false, addable: true },
    ComponentDescriptor { kind: ComponentKind::Visibility, label: "Visibility", category: "Core", properties: VISIBILITY_PROPERTIES, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::Parent, label: "Parent", category: "Core", properties: PARENT_PROPERTIES, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::Camera, label: "Camera", category: "Rendering", properties: CAMERA_PROPERTIES, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::Light, label: "Light", category: "Rendering", properties: LIGHT_PROPERTIES, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::Mesh, label: "Mesh", category: "Rendering", properties: EMPTY, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::Material, label: "Material", category: "Rendering", properties: EMPTY, removable: true, addable: true },
    ComponentDescriptor { kind: ComponentKind::EditorEntity, label: "Editor Entity", category: "Editor", properties: EMPTY, removable: false, addable: false },
];

#[derive(Resource, Debug, Clone)]
pub struct ComponentRegistry {
    descriptors: BTreeMap<ComponentKind, ComponentDescriptor>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        let mut descriptors = BTreeMap::new();
        for descriptor in DESCRIPTORS {
            descriptors.insert(descriptor.kind, descriptor.clone());
        }
        Self { descriptors }
    }
}

impl ComponentRegistry {
    pub fn descriptor(&self, kind: ComponentKind) -> Option<&ComponentDescriptor> {
        self.descriptors.get(&kind)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentDescriptor> {
        self.descriptors.values()
    }

    pub fn by_category(&self, category: &str) -> impl Iterator<Item = &ComponentDescriptor> {
        self.descriptors.values().filter(move |item| item.category == category)
    }

    pub fn detect_entity(&self, world: &World, entity: Entity) -> Vec<ComponentKind> {
        let mut result = Vec::new();
        if world.get::<Transform>(entity).is_some() {
            result.push(ComponentKind::Transform);
        }
        if world.get::<Name>(entity).is_some() {
            result.push(ComponentKind::Name);
        }
        if world.get::<Visibility>(entity).is_some() {
            result.push(ComponentKind::Visibility);
        }
        if world.get::<crate::EditorParent>(entity).is_some() {
            result.push(ComponentKind::Parent);
        }
        if world.get::<crate::viewport::EditorEntity>(entity).is_some() {
            result.push(ComponentKind::EditorEntity);
        }
        if world.get::<Camera>(entity).is_some() {
            result.push(ComponentKind::Camera);
        }
        if world.get::<DirectionalLight>(entity).is_some()
            || world.get::<PointLight>(entity).is_some()
            || world.get::<SpotLight>(entity).is_some()
        {
            result.push(ComponentKind::Light);
        }
        result
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for descriptor in self.descriptors.values() {
            if descriptor.label.is_empty() {
                errors.push(format!("component {:?} has empty label", descriptor.kind));
            }
            for property in descriptor.properties {
                if property.name.is_empty() || property.label.is_empty() {
                    errors.push(format!("component {:?} contains invalid property", descriptor.kind));
                }
                if property.editable && matches!(property.kind, PropertyKind::ReadOnly) {
                    errors.push(format!("property {} is editable but read-only", property.name));
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    pub fn component_count(&self) -> usize {
        self.descriptors.len()
    }

    pub fn property_count(&self) -> usize {
        self.descriptors.values().map(|descriptor| descriptor.properties.len()).sum()
    }
}

pub fn install_component_registry(app: &mut App) {
    app.init_resource::<ComponentRegistry>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_core_components() {
        let registry = ComponentRegistry::default();
        assert!(registry.descriptor(ComponentKind::Transform).is_some());
        assert!(registry.descriptor(ComponentKind::Visibility).is_some());
        assert!(registry.component_count() >= 9);
    }

    #[test]
    fn registry_metadata_is_valid() {
        assert!(ComponentRegistry::default().validate().is_ok());
    }
}
