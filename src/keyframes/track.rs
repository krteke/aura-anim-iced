use crate::property::{PropertyValue, RawPropertySpec};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PropertyTrack {
    spec: RawPropertySpec,
    samples: Vec<PropertySample>,
}

impl PropertyTrack {
    pub(super) fn new(spec: RawPropertySpec, samples: Vec<PropertySample>) -> Self {
        Self { spec, samples }
    }

    pub(super) fn samples(&self) -> &[PropertySample] {
        &self.samples
    }

    pub(super) fn add_sample(&mut self, offset: f32, value: PropertyValue) {
        self.samples.push(PropertySample::new(offset, value));
    }

    pub(super) fn spec(&self) -> &RawPropertySpec {
        &self.spec
    }

    pub(super) fn sort_samples(&mut self) {
        self.samples.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PropertySample {
    offset: f32,
    value: PropertyValue,
}

impl PropertySample {
    pub(super) fn new(offset: f32, value: PropertyValue) -> Self {
        Self { offset, value }
    }

    pub(super) fn offset(&self) -> f32 {
        self.offset
    }

    pub(super) fn value(&self) -> PropertyValue {
        self.value
    }
}
