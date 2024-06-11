use nanorand::{Rng, WyRand};

#[derive(Debug)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub fn region_is_empty(
    table: &[u32],
    table_width: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> bool {
    let tl = table[y * table_width + x];
    let tr = table[y * table_width + x + width];

    let bl = table[(y + height) * table_width + x];
    let br = table[(y + height) * table_width + x + width];

    tl as i32 + br as i32 - tr as i32 - bl as i32 == 0
}

/// 在图片寻找位置写字
pub fn find_space_for_rect(
    table: &[u32],
    table_width: u32,
    table_height: u32,
    rect: &Rect,
    rng: &mut WyRand,
) -> Option<Point> {
    let max_x = table_width - rect.width;
    let max_y = table_height - rect.height;

    let mut available_points: u32 = 0;
    let mut random_pont = None;

    // column based
    for y in 0..max_y {
        for x in 0..max_x {
            let empty = region_is_empty(
                table,
                table_width as usize,
                x as usize,
                y as usize,
                rect.width as usize,
                rect.height as usize,
            );
            if empty {
                let random_num = rng.generate_range(0..=available_points);
                if random_num == available_points {
                    random_pont = Some(Point { x, y });
                }
                available_points += 1;
            }
        }
    }

    random_pont
}

/// https://blog.demofox.org/2018/04/16/prefix-sums-and-summed-area-tables/
pub fn to_summed_area_table(table: &mut [u32], width: usize, start_row: usize) {
    let mut prev_row = vec![0; width];
    table
        .chunks_exact_mut(width)
        .skip(start_row)
        .for_each(|row| {
            let mut sum = 0;
            row.iter_mut()
                .zip(prev_row.iter())
                .for_each(|(el, prev_row_el)| {
                    let originval_value = *el;
                    *el += sum + prev_row_el;
                    sum += originval_value;
                });

            prev_row.clone_from_slice(row)
        });
}
