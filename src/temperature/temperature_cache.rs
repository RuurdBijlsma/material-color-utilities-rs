use crate::hct::hct_color::Hct;
use crate::utils::color_utils::Argb;
use crate::utils::math_utils::MathUtils;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

type OnceLockHashMap<K, V> = OnceLock<RwLock<HashMap<K, V>>>;
static COMPLEMENT_CACHE: OnceLockHashMap<Argb, Hct> = OnceLock::new();
static HCTS_BY_HUE_CACHE: OnceLockHashMap<(u64, u64), Vec<Hct>> = OnceLock::new();
static HCTS_BY_TEMP_CACHE: OnceLockHashMap<(u64, u64), Vec<Hct>> = OnceLock::new();
static TEMPS_BY_HCT_CACHE: OnceLockHashMap<(u64, u64), HashMap<Argb, f64>> = OnceLock::new();

pub struct TemperatureCache {
    input: Hct,
}

impl TemperatureCache {
    #[must_use]
    pub const fn new(input: Hct) -> Self {
        Self { input }
    }

    fn get_global_map<K, V>(lock: &OnceLock<RwLock<HashMap<K, V>>>) -> &RwLock<HashMap<K, V>>
    where
        K: Eq + std::hash::Hash,
    {
        lock.get_or_init(|| RwLock::new(HashMap::new()))
    }

    #[must_use]
    pub fn complement(&self) -> Hct {
        let cache = Self::get_global_map(&COMPLEMENT_CACHE);
        let key = self.input.to_argb();

        // 1. Try to read from cache
        if let Ok(map) = cache.read()
            && let Some(&complement) = map.get(&key) {
                return complement;
            }

        // 2. Compute
        let coldest = self.coldest();
        let coldest_hue = coldest.hue();
        let coldest_temp = self.get_temp(&coldest);

        let warmest = self.warmest();
        let warmest_hue = warmest.hue();
        let warmest_temp = self.get_temp(&warmest);

        let range = warmest_temp - coldest_temp;
        let start_hue_is_coldest_to_warmest =
            Self::is_between(self.input.hue(), coldest_hue, warmest_hue);

        let (start_hue, end_hue) = if start_hue_is_coldest_to_warmest {
            (warmest_hue, coldest_hue)
        } else {
            (coldest_hue, warmest_hue)
        };

        let direction_of_rotation = 1.0;
        let mut smallest_error = 1000.0;

        let hcts_by_hue = self.hcts_by_hue();
        let mut answer = hcts_by_hue[self.input.hue().round() as usize % 360];

        let complement_relative_temp = 1.0 - self.get_relative_temperature(&self.input);

        let mut hue_addend = 0.0;
        while hue_addend <= 360.0 {
            let hue =
                MathUtils::sanitize_degrees_double(start_hue + direction_of_rotation * hue_addend);
            if !Self::is_between(hue, start_hue, end_hue) {
                hue_addend += 1.0;
                continue;
            }

            let possible_answer = hcts_by_hue[hue.round() as usize % 360];
            let relative_temp = (self.get_temp(&possible_answer) - coldest_temp) / range;
            let error = (complement_relative_temp - relative_temp).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible_answer;
            }
            hue_addend += 1.0;
        }

        // 3. Write to cache
        if let Ok(mut map) = cache.write() {
            map.insert(key, answer);
        }
        answer
    }

    /// 5 colors that pair well with the input color.
    ///
    /// The colors are equidistant in temperature and adjacent in hue.
    #[must_use]
    pub fn get_analogous_colors(&self) -> Vec<Hct> {
        self.get_analogous_colors_with_options(5, 12)
    }

    /// A set of colors with differing hues, equidistant in temperature.
    ///
    /// In art, this is usually described as a set of 5 colors on a color wheel divided into 12
    /// sections. This method allows provision of either of those values.
    ///
    /// Behavior is undefined when count or divisions is 0. When divisions < count, colors repeat.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of colors to return, includes the input color.
    /// * `divisions` - The number of divisions on the color wheel.
    #[must_use]
    pub fn get_analogous_colors_with_options(&self, count: usize, divisions: usize) -> Vec<Hct> {
        // The starting hue is the hue of the input color.
        let start_hue = self.input.hue().round() as i32;
        let hcts_by_hue = self.hcts_by_hue();
        let start_hct = hcts_by_hue[MathUtils::sanitize_degrees_int(start_hue) as usize % 360];
        let mut last_temp = self.get_relative_temperature(&start_hct);

        let mut all_colors: Vec<Hct> = Vec::new();
        all_colors.push(start_hct);

        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = MathUtils::sanitize_degrees_int(start_hue + i);
            let hct = hcts_by_hue[hue as usize % 360];
            let temp = self.get_relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            last_temp = temp;
            absolute_total_temp_delta += temp_delta;
        }

        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / divisions as f64;
        let mut total_temp_delta = 0.0;
        last_temp = self.get_relative_temperature(&start_hct);

        while all_colors.len() < divisions {
            let hue = MathUtils::sanitize_degrees_int(start_hue + hue_addend);
            let hct = hcts_by_hue[hue as usize % 360];
            let temp = self.get_relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            total_temp_delta += temp_delta;

            let mut desired_total_temp_delta_for_index = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
            let mut index_addend = 1;

            // Keep adding this hue to the answers until its temperature is
            // insufficient. This ensures consistent behavior when there aren't
            // `divisions` discrete steps between 0 and 360 in hue with `temp_step`
            // delta in temperature between them.
            while index_satisfied && all_colors.len() < divisions {
                all_colors.push(hct);
                desired_total_temp_delta_for_index =
                    (all_colors.len() + index_addend) as f64 * temp_step;
                index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;

            if hue_addend > 360 {
                while all_colors.len() < divisions {
                    all_colors.push(hct);
                }
                break;
            }
        }

        let mut answers: Vec<Hct> = Vec::new();
        answers.push(self.input);

        let ccw_count = ((count as f64 - 1.0) / 2.0).floor() as usize;
        for i in 1..=ccw_count {
            let mut index = 0i32 - i as i32;
            while index < 0 {
                index += all_colors.len() as i32;
            }
            let idx = (index as usize) % all_colors.len();
            answers.insert(0, all_colors[idx]);
        }

        let cw_count = count - ccw_count - 1;
        for i in 1..=cw_count {
            let index = i;
            let idx = index % all_colors.len();
            answers.push(all_colors[idx]);
        }

        answers
    }

    /// Temperature relative to all colors with the same chroma and tone.
    ///
    /// @param hct HCT to find the relative temperature of.
    /// @return Value on a scale from 0 to 1.
    #[must_use]
    pub fn get_relative_temperature(&self, hct: &Hct) -> f64 {
        let coldest_temp = self.get_temp(&self.coldest());
        let warmest_temp = self.get_temp(&self.warmest());

        let range = warmest_temp - coldest_temp;
        let hct_temp = self.get_temp(hct);

        let difference_from_coldest = hct_temp - coldest_temp;

        // Handle when there's no difference in temperature between warmest and
        // coldest: for example, at T100, only one color is available, white.
        if range == 0.0 {
            0.5
        } else {
            difference_from_coldest / range
        }
    }

    fn get_temp(&self, hct: &Hct) -> f64 {
        let chroma_tone_key = (self.input.chroma().to_bits(), self.input.tone().to_bits());
        let cache = Self::get_global_map(&TEMPS_BY_HCT_CACHE);

        if let Ok(map) = cache.read()
            && let Some(temps) = map.get(&chroma_tone_key)
                && let Some(&temp) = temps.get(&hct.to_argb()) {
                    return temp;
                }

        // Fallback to recalculating (this is rare if temps_by_hct() was called)
        Self::raw_temperature(hct)
    }

    fn coldest(&self) -> Hct {
        self.hcts_by_temp()[0]
    }

    /// Warmest color with same chroma and tone as input.
    fn warmest(&self) -> Hct {
        let hcts = self.hcts_by_temp();
        hcts[hcts.len() - 1]
    }

    fn hcts_by_hue(&self) -> Vec<Hct> {
        let key = (self.input.chroma().to_bits(), self.input.tone().to_bits());
        let cache = Self::get_global_map(&HCTS_BY_HUE_CACHE);

        if let Ok(map) = cache.read()
            && let Some(hcts) = map.get(&key) {
                return hcts.clone();
            }

        let mut hcts = Vec::with_capacity(360);
        for i in 0..360 {
            hcts.push(Hct::from(f64::from(i), self.input.chroma(), self.input.tone()));
        }

        if let Ok(mut map) = cache.write() {
            map.insert(key, hcts.clone());
        }
        hcts
    }

    fn hcts_by_temp(&self) -> Vec<Hct> {
        let key = (self.input.chroma().to_bits(), self.input.tone().to_bits());
        let cache = Self::get_global_map(&HCTS_BY_TEMP_CACHE);

        if let Ok(map) = cache.read()
            && let Some(hcts) = map.get(&key) {
                return hcts.clone();
            }

        let mut hcts = self.hcts_by_hue();
        hcts.push(self.input);

        // We ensure temps_by_hct is populated first to make sorting efficient
        let temps = self.temps_by_hct_internal();
        hcts.sort_by(|a, b| {
            let temp_a = temps.get(&a.to_argb()).unwrap_or(&0.0);
            let temp_b = temps.get(&b.to_argb()).unwrap_or(&0.0);
            temp_a
                .partial_cmp(temp_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Ok(mut map) = cache.write() {
            map.insert(key, hcts.clone());
        }
        hcts
    }

    /// Internal helper to access/populate the global temp map
    fn temps_by_hct_internal(&self) -> HashMap<Argb, f64> {
        let key = (self.input.chroma().to_bits(), self.input.tone().to_bits());
        let cache = Self::get_global_map(&TEMPS_BY_HCT_CACHE);

        if let Ok(map) = cache.read()
            && let Some(temps) = map.get(&key) {
                return temps.clone();
            }

        let mut all_hcts = self.hcts_by_hue();
        all_hcts.push(self.input);
        let mut temperatures_by_hct = HashMap::new();
        for hct in all_hcts {
            temperatures_by_hct.insert(hct.to_argb(), Self::raw_temperature(&hct));
        }

        if let Ok(mut map) = cache.write() {
            map.insert(key, temperatures_by_hct.clone());
        }
        temperatures_by_hct
    }

    #[must_use]
    pub fn raw_temperature(color: &Hct) -> f64 {
        let lab = color.to_argb().to_lab();
        let hue = MathUtils::sanitize_degrees_double(lab.b.atan2(lab.a).to_degrees());
        let chroma = lab.a.hypot(lab.b);

        (0.02 * chroma.powf(1.07)).mul_add(
            MathUtils::sanitize_degrees_double(hue - 50.0)
                .to_radians()
                .cos(),
            -0.5,
        )
    }

    fn is_between(angle: f64, a: f64, b: f64) -> bool {
        if a < b {
            a <= angle && angle <= b
        } else {
            a <= angle || angle <= b
        }
    }
}
