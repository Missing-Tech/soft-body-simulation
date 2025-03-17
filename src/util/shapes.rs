use bevy::math::Vec3;

pub fn get_trapezoidal_area(points: &[Vec3]) -> f32 {
    let mut area: f32 = 0.;

    for i in 0..points.len() {
        let current = points[i];
        let next = if i == points.len() - 1 {
            points[0]
        } else {
            points[i + 1]
        };

        area += (current.x - next.x) * (current.y + next.y) / 2.;
    }

    return area.abs();
}
