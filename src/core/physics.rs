pub fn point_in_circle(point: egui::Pos2, circle_center: egui::Pos2, radius: f32) -> bool {
    let dx = point.x - circle_center.x;
    let dy = point.y - circle_center.y;
    (dx * dx + dy * dy) <= radius * radius
}

pub fn circle_collision(center1: egui::Pos2, radius1: f32, center2: egui::Pos2, radius2: f32) -> bool {
    let dx = center1.x - center2.x;
    let dy = center1.y - center2.y;
    let distance_squared = dx * dx + dy * dy;
    let radius_sum = radius1 + radius2;
    distance_squared <= radius_sum * radius_sum
}

pub fn force_circles_away(center1: egui::Pos2, radius1: f32, center2: egui::Pos2, radius2: f32) -> (egui::Vec2, egui::Vec2) {
    let dx = center1.x - center2.x;
    let dy = center1.y - center2.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let overlap = radius1 + radius2 - distance;

    if overlap > 0.0 {
        let direction = egui::Vec2::new(dx / distance, dy / distance);
        let force = direction * overlap / 2.0; // Split the force between the two circles
        return (force, -force); // Return forces for both circles
    }

    (egui::Vec2::ZERO, egui::Vec2::ZERO) // No collision, no force
}