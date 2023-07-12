use lazy_static::lazy_static;
use ordered_float::OrderedFloat;

const GESTURE_RESOLUTION: usize = 128;

pub struct PreparedGesture([(f32, f32); GESTURE_RESOLUTION]);

impl PreparedGesture {
    pub fn distance(&self, other: &PreparedGesture) -> f32 {
        let mut distance = 0.0;

        for sample_index in 0..GESTURE_RESOLUTION {
            distance += (self.0[sample_index].0 - other.0[sample_index].0).abs();
            distance += (self.0[sample_index].1 - other.0[sample_index].1).abs();
        }

        distance
    }

    pub fn from_positions(input_positions: &[(f32, f32)]) -> Option<PreparedGesture> {
        // Deduplicate cursor positions to avoid zero-length segments
        let dedup_positions = {
            let mut dedup_positions = input_positions.to_owned();

            dedup_positions.dedup();
            if dedup_positions.len() < 2 {
                return None;
            }

            dedup_positions
        };

        // Normalize path bounds to ((-1.0..=1.0), (-1.0..=1.0))
        let normalized_path = {
            let x_min = dedup_positions.iter().map(|(x, _)| *x).reduce(f32::min).unwrap_or(0.0);
            let x_max = dedup_positions.iter().map(|(x, _)| *x).reduce(f32::max).unwrap_or(0.0);
            let y_min = dedup_positions.iter().map(|(_, y)| *y).reduce(f32::min).unwrap_or(0.0);
            let y_max = dedup_positions.iter().map(|(_, y)| *y).reduce(f32::max).unwrap_or(0.0);

            let (x_center, x_half_size) = ((x_max + x_min) / 2.0, (x_max - x_min) / 2.0);
            let (y_center, y_half_size) = ((y_max + y_min) / 2.0, (y_max - y_min) / 2.0);

            let half_size = x_half_size.max(y_half_size);
            assert_ne!(half_size, 0.0);

            dedup_positions
                .into_iter()
                .map(|(x, y)| ((x - x_center) / half_size, (y - y_center) / half_size))
                .collect::<Vec<(f32, f32)>>()
        };

        // Resample path into N uniformly spaced samples
        let resampled_path = {
            let mut total_distance = 0.0;

            let segments = normalized_path
                .windows(2)
                .map(|window| (window[0], window[1]))
                .map(|(start_point, end_point)| {
                    let segment_length =
                        ((end_point.0 - start_point.0).powi(2) + (end_point.1 - start_point.1).powi(2)).sqrt();
                    (start_point, end_point, segment_length)
                })
                .map(|(start_point, end_point, segment_length)| {
                    let result = (start_point, end_point, segment_length, total_distance);
                    total_distance += segment_length;
                    result
                })
                .collect::<Vec<_>>();

            (0..GESTURE_RESOLUTION)
                .map(|i| i as f32 / (GESTURE_RESOLUTION - 1) as f32)
                .map(|t| {
                    segments
                        .iter()
                        .find(|(_start_point, _end_point, segment_length, distance_from_start)| {
                            (*distance_from_start..=distance_from_start + segment_length)
                                .contains(&(t * total_distance))
                        })
                        .map(|(start_point, end_point, segment_length, distance_from_start)| {
                            let t = ((t * total_distance) - distance_from_start) / segment_length;

                            (
                                start_point.0 * (1.0 - t) + end_point.0 * t,
                                start_point.1 * (1.0 - t) + end_point.1 * t,
                            )
                        })
                        .unwrap()
                })
                .collect::<Vec<(f32, f32)>>()
        };

        Some(PreparedGesture(resampled_path.try_into().unwrap()))
    }
}

pub fn gesture_from_positions(input_positions: &[(f32, f32)]) -> Option<&'static str> {
    let input_gesture = PreparedGesture::from_positions(input_positions)?;

    let (gesture_name, _) = GESTURES
        .iter()
        .min_by_key(|(_, gesture)| OrderedFloat(input_gesture.distance(gesture)))
        .unwrap();

    Some(gesture_name)
}

macro_rules! gesture {
    [$($position:expr),* $(,)?] => {
        PreparedGesture::from_positions(&[$(($position),)*]).unwrap()
    }
}

#[rustfmt::skip]
lazy_static! {
    static ref GESTURES: Vec<(&'static str, PreparedGesture)> = vec![
        ("down",           gesture![(   0.0,    0.0), (   0.0,  100.0),                                   ]),
        ("down-left",      gesture![(   0.0,    0.0), (   0.0,  100.0), (-100.0,  100.0),                 ]),
        ("down-right",     gesture![(   0.0,    0.0), (   0.0,  100.0), ( 100.0,  100.0),                 ]),
        ("down-up",        gesture![(   0.0,    0.0), (   0.0,  100.0), (   0.0,    0.0),                 ]),

        ("left",           gesture![(   0.0,    0.0), (-100.0,    0.0),                                   ]),
        ("left-down",      gesture![(   0.0,    0.0), (-100.0,    0.0), (-100.0,  100.0),                 ]),
        ("left-right",     gesture![(   0.0,    0.0), (-100.0,    0.0), (   0.0,    0.0),                 ]),
        ("left-up",        gesture![(   0.0,    0.0), (-100.0,    0.0), (-100.0, -100.0),                 ]),

        ("right",          gesture![(   0.0,    0.0), ( 100.0,    0.0),                                   ]),
        ("right-down",     gesture![(   0.0,    0.0), ( 100.0,    0.0), ( 100.0,  100.0),                 ]),
        ("right-left",     gesture![(   0.0,    0.0), ( 100.0,    0.0), (   0.0,    0.0),                 ]),
        ("right-up",       gesture![(   0.0,    0.0), ( 100.0,    0.0), ( 100.0, -100.0),                 ]),

        ("up",             gesture![(   0.0,    0.0), (   0.0, -100.0),                                   ]),
        ("up-down",        gesture![(   0.0,    0.0), (   0.0, -100.0), (   0.0,    0.0),                 ]),
        ("up-left",        gesture![(   0.0,    0.0), (   0.0, -100.0), (-100.0, -100.0),                 ]),
        ("up-right",       gesture![(   0.0,    0.0), (   0.0, -100.0), ( 100.0, -100.0),                 ]),

        ("diag-downleft",  gesture![(   0.0,    0.0), (-100.0,  100.0),                                   ]),
        ("diag-downright", gesture![(   0.0,    0.0), ( 100.0,  100.0),                                   ]),
        ("diag-upleft",    gesture![(   0.0,    0.0), (-100.0, -100.0),                                   ]),
        ("diag-upright",   gesture![(   0.0,    0.0), ( 100.0, -100.0),                                   ]),

        ("rectangle",      gesture![(   0.0,    0.0), ( 100.0,    0.0), ( 100.0,  100.0), (   0.0,  100.0)]),
        ("rectangle",      gesture![(   0.0,    0.0), (   0.0,  100.0), ( 100.0,  100.0), ( 100.0,    0.0)]),

        ("arrow-up",       gesture![(   0.0,  100.0), (  50.0,    0.0), ( 100.0,  100.0),                 ]),
        ("arrow-down",     gesture![(   0.0,    0.0), (  50.0,  100.0), ( 100.0,    0.0),                 ]),
        ("arrow-left",     gesture![( 100.0,    0.0), (   0.0,   50.0), ( 100.0,  100.0),                 ]),
        ("arrow-right",    gesture![(   0.0,    0.0), ( 100.0,   50.0), (   0.0,  100.0),                 ]),

        ("tri-up",         gesture![(   0.0,  100.0), (  50.0,    0.0), ( 100.0,  100.0), (   0.0,  100.0)]),
        ("tri-down",       gesture![(   0.0,    0.0), (  50.0,  100.0), ( 100.0,    0.0), (   0.0,    0.0)]),
        ("tri-left",       gesture![( 100.0,    0.0), (   0.0,   50.0), ( 100.0,  100.0), ( 100.0,    0.0)]),
        ("tri-right",      gesture![(   0.0,    0.0), ( 100.0,   50.0), (   0.0,  100.0), (   0.0,    0.0)]),

        ("n",              gesture![(   0.0,  100.0), (   0.0,    0.0), ( 100.0,  100.0), ( 100.0,    0.0)]),
        ("t",              gesture![(   0.0,    0.0), ( 100.0,    0.0), (  50.0,    0.0), (  50.0,  100.0)]),
        ("x",              gesture![(   0.0,    0.0), ( 100.0,  100.0), ( 100.0,    0.0), (   0.0,  100.0)]),
        ("z",              gesture![(   0.0,    0.0), ( 100.0,    0.0), (   0.0,  100.0), ( 100.0,  100.0)]),
    ];
}
