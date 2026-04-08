use crate::hittable::Hittable;

pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

pub struct SceneIter<'a> {
    inner: std::slice::Iter<'a, Box<dyn Hittable>>,
}

impl Scene {
    pub fn new(capacity: usize) -> Self {
        let vec = Vec::with_capacity(capacity);

        Self { objects: vec }
    }

    pub fn add_object<T: Hittable + 'static>(&mut self, obj: T) {
        self.objects.push(Box::new(obj));
    }

    pub fn iter(&self) -> SceneIter<'_> {
        SceneIter {
            inner: self.objects.iter(),
        }
    }
}

impl<'a> Iterator for SceneIter<'a> {
    type Item = &'a dyn Hittable;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Box::as_ref)
    }
}

impl<'a> IntoIterator for &'a Scene {
    type Item = &'a dyn Hittable;
    type IntoIter = SceneIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
