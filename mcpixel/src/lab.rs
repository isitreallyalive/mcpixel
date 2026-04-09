use palette::{IntoColor, Lab, Srgb};

pub(crate) fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let [l1, a1, b1] = a;
    let [l2, a2, b2] = b;
    (l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)
}

pub(crate) fn from_rgb([r, g, b]: [u8; 3]) -> [f32; 3] {
    let lab: Lab = Srgb::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.).into_color();
    [lab.l, lab.a, lab.b]
}
