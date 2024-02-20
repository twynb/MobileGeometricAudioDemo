use wav::BitDepth;

#[allow(clippy::module_name_repetitions)]
pub fn to_impulse_response(results: &[(f64, u32)], number_of_rays: u32) -> Vec<f64> {
    let buf_size = results
        .iter()
        .max_by_key(|result| result.1)
        .unwrap_or(&(0f64, 0))
        .1 as usize
        + 1;
    let mut impulse_response_buffer = vec![0f64; buf_size];
    for result in results {
        impulse_response_buffer[result.1 as usize] += result.0;
    }
    let number_of_rays_float = number_of_rays as f64;
    impulse_response_buffer
        .iter()
        .map(|val| val / number_of_rays_float)
        .collect()
}

pub fn apply_to_data(impulse_response: &[Vec<f64>], data: &BitDepth) -> BitDepth {
    match data {
        BitDepth::Eight(stream) => {
            BitDepth::Eight(apply_to_data_internal(impulse_response, stream))
        }
        BitDepth::Sixteen(stream) => {
            BitDepth::Sixteen(apply_to_data_internal(impulse_response, stream))
        }
        BitDepth::TwentyFour(stream) => {
            BitDepth::TwentyFour(apply_to_data_internal(impulse_response, stream))
        }
        BitDepth::ThirtyTwoFloat(stream) => {
            BitDepth::ThirtyTwoFloat(apply_to_data_internal(impulse_response, stream))
        }
        BitDepth::Empty => BitDepth::Empty,
    }
}

fn apply_to_data_internal<T: num::Num + num::NumCast + Clone + Copy>(
    impulse_response: &[Vec<f64>],
    data: &[T],
) -> Vec<T> {
    let max_t60 = impulse_response
        .iter()
        .max_by_key(|result| result.len())
        .unwrap_or(&vec![])
        .len()
        + 1;
    let mut buffer = vec![T::zero(); max_t60 + data.len()];
    for (index, sample) in data.iter().enumerate() {
        let response = &impulse_response[index];
        for (idx, value) in response.iter().enumerate() {
            buffer[index + idx] = buffer[index + idx]
                + num::cast::<f64, T>(num::cast::<T, f64>(*sample).unwrap() * value).unwrap();
        }
    }
    buffer
}

pub fn apply_to_sample<T: num::Num + num::NumCast + Clone + Copy>(
    impulse_response: &[f64],
    sample: T,
    index: usize,
    scaling_factor: f64,
) -> Vec<f64> {
    let mut buffer = vec![0f64; impulse_response.len() + index + 1];
    for (idx, value) in impulse_response.iter().enumerate() {
        buffer[idx] = num::cast::<T, f64>(sample).unwrap() * value * scaling_factor;
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::to_impulse_response;

    #[test]
    fn empty_result_to_impulse_response() {
        let input: Vec<(f64, u32)> = vec![];
        let result = to_impulse_response(&input, 10000);
        assert_eq!(vec![0f64], result)
    }

    #[test]
    fn single_result_to_impulse_response() {
        let input = vec![(1.0f64, 90)];
        let mut expected = vec![0f64; 91];
        expected[90] = 0.0001f64;
        assert_eq!(expected, to_impulse_response(&input, 10000))
    }

    #[test]
    fn duplicate_result_to_impulse_response() {
        let input = vec![(1.0f64, 90), (0.5f64, 90)];
        let mut expected = vec![0f64; 91];
        expected[90] = 0.00015f64;
        assert_eq!(expected, to_impulse_response(&input, 10000))
    }
}
