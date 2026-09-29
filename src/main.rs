use macroquad::prelude::*;
use rayon::prelude::*;

const ALIVE: u8 = 1;
const DEAD: u8 = 0;
const WINDOW_WIDTH: u32 = 1_600;
const WINDOW_HEIGHT: u32 = 900;
const WORLD_WIDTH: usize = 8_000;
const WORLD_HEIGHT: usize = 4_500;

struct Grid {
    width: usize,
    height: usize,
    stride: usize,
    cells: Vec<u8>,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        let stride = width + 2;
        Self {
            width,
            height,
            stride,
            cells: vec![DEAD; (height + 2) * stride],
        }
    }

    fn set_alive(&mut self, x: usize, y: usize) {
        self.cells[(y + 1) * self.stride + x + 1] = ALIVE;
    }

    #[cfg(test)]
    fn alive_cells(&self) -> Vec<(usize, usize)> {
        let mut alive = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[(y + 1) * self.stride + x + 1] == ALIVE {
                    alive.push((x, y));
                }
            }
        }
        alive
    }
}

fn step_generation(current: &Grid, next: &mut Grid) {
    debug_assert_eq!(current.width, next.width);
    debug_assert_eq!(current.height, next.height);

    next.cells
        .par_chunks_mut(current.stride)
        .enumerate()
        .skip(1)
        .take(current.height)
        .for_each(|(y, next_row)| {
            let row = y * current.stride;
            for x in 1..=current.width {
                let neighbors = current.cells[row - current.stride + x - 1]
                    + current.cells[row - current.stride + x]
                    + current.cells[row - current.stride + x + 1]
                    + current.cells[row + x - 1]
                    + current.cells[row + x + 1]
                    + current.cells[row + current.stride + x - 1]
                    + current.cells[row + current.stride + x]
                    + current.cells[row + current.stride + x + 1];
                next_row[x] =
                    u8::from(neighbors == 3 || (current.cells[row + x] == ALIVE && neighbors == 2));
            }
        });
}

fn update_image(image: &mut Image, grid: &Grid, view: ViewTransform) {
    let image_width = image.width();
    let image_height = image.height();
    let pixels = image.get_image_data_mut();

    for y in 0..image_height {
        let world_y = ((y as f32 - view.offset.y) / view.zoom).floor() as isize;
        for x in 0..image_width {
            let world_x = ((x as f32 - view.offset.x) / view.zoom).floor() as isize;
            let pixel = &mut pixels[y * image_width + x];
            *pixel = if world_x >= 0
                && world_x < grid.width as isize
                && world_y >= 0
                && world_y < grid.height as isize
                && grid.cells[(world_y as usize + 1) * grid.stride + world_x as usize + 1] == ALIVE
            {
                BLACK.into()
            } else {
                WHITE.into()
            };
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ViewTransform {
    offset: Vec2,
    zoom: f32,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl ViewTransform {
    const MAX_ZOOM: f32 = 4.0;

    fn minimum_zoom(world_size: Vec2, viewport_size: Vec2) -> f32 {
        (viewport_size.x / world_size.x).max(viewport_size.y / world_size.y)
    }

    fn screen_to_world(self, screen_position: Vec2) -> Vec2 {
        (screen_position - self.offset) / self.zoom
    }

    fn zoom_at(&mut self, screen_position: Vec2, factor: f32) {
        let world_position = self.screen_to_world(screen_position);
        self.zoom = (self.zoom * factor).min(Self::MAX_ZOOM);
        self.offset = screen_position - world_position * self.zoom;
    }

    fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    fn clamp_to_bounds(&mut self, world_size: Vec2, viewport_size: Vec2) {
        let scaled_world = world_size * self.zoom;
        self.offset.x = self.offset.x.clamp(viewport_size.x - scaled_world.x, 0.0);
        self.offset.y = self.offset.y.clamp(viewport_size.y - scaled_world.y, 0.0);
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Life".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        platform: miniquad::conf::Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let width = WORLD_WIDTH;
    let height = WORLD_HEIGHT;

    let mut current = Grid::new(width, height);
    let mut next = Grid::new(width, height);
    for y in 0..height {
        for x in 0..width {
            if rand::gen_range(0, 5) == 0 {
                current.set_alive(x, y);
            }
        }
    }

    let mut image = Image::gen_image_color(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, WHITE);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    let mut view = ViewTransform::default();
    let world_size = vec2(width as f32, height as f32);
    let viewport_size = vec2(screen_width(), screen_height());
    let minimum_zoom = ViewTransform::minimum_zoom(world_size, viewport_size);
    let mut previous_mouse_position = vec2(mouse_position().0, mouse_position().1);
    let mut dragging = false;

    loop {
        let mouse_position = vec2(mouse_position().0, mouse_position().1);
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            view.zoom_at(mouse_position, 1.1_f32.powf(wheel_y));
            view.zoom = view.zoom.clamp(minimum_zoom, ViewTransform::MAX_ZOOM);
        }

        if is_mouse_button_down(MouseButton::Left) {
            if dragging {
                view.pan(mouse_position - previous_mouse_position);
            }
            dragging = true;
        } else {
            dragging = false;
        }
        view.clamp_to_bounds(world_size, viewport_size);
        previous_mouse_position = mouse_position;

        step_generation(&current, &mut next);
        std::mem::swap(&mut current, &mut next);
        update_image(&mut image, &current, view);
        texture.update(&image);
        clear_background(WHITE);
        draw_texture(&texture, 0., 0., WHITE);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 25.0, 24.0, RED);

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use macroquad::prelude::vec2;

    use super::{Grid, ViewTransform, step_generation};

    #[test]
    fn zoom_keeps_the_world_point_under_the_cursor() {
        let mut view = ViewTransform::default();
        let cursor = vec2(320.0, 180.0);
        let world_point = view.screen_to_world(cursor);

        view.zoom_at(cursor, 2.0);

        assert!((view.screen_to_world(cursor) - world_point).length() < 0.001);
    }

    #[test]
    fn pan_moves_the_view_by_the_drag_delta() {
        let mut view = ViewTransform::default();

        view.pan(vec2(25.0, -10.0));

        assert_eq!(view.offset, vec2(25.0, -10.0));
    }

    #[test]
    fn minimum_zoom_is_large_enough_to_cover_the_window() {
        assert_eq!(
            ViewTransform::minimum_zoom(vec2(16_000.0, 9_000.0), vec2(1_600.0, 900.0)),
            0.1
        );
    }

    #[test]
    fn view_offset_is_clamped_to_the_world_bounds() {
        let mut view = ViewTransform {
            offset: vec2(100.0, -20_000.0),
            zoom: 1.0,
        };

        view.clamp_to_bounds(vec2(16_000.0, 9_000.0), vec2(1_600.0, 900.0));

        assert_eq!(view.offset, vec2(0.0, -8_100.0));
    }

    #[test]
    fn step_generation_preserves_the_blinker_oscillator() {
        let mut current = Grid::new(5, 5);
        current.set_alive(2, 1);
        current.set_alive(2, 2);
        current.set_alive(2, 3);
        let mut next = Grid::new(5, 5);
        step_generation(&current, &mut next);
        assert_eq!(next.alive_cells(), &[(1, 2), (2, 2), (3, 2)]);
    }

    #[test]
    fn step_generation_keeps_the_outer_boundary_dead() {
        let mut current = Grid::new(3, 3);
        current.set_alive(0, 0);
        current.set_alive(0, 1);
        let mut next = Grid::new(3, 3);
        step_generation(&current, &mut next);
        assert!(next.alive_cells().is_empty());
    }
}
