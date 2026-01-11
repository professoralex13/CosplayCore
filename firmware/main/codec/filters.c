#include "filters.h"
#include <math.h>
#include <string.h>

#define PI 3.14159265359f

// Initialize a biquad filter with given coefficients
static void biquad_init(biquad_filter_t *filter, float b0, float b1, float b2,
                        float a0, float a1, float a2) {
    filter->b0 = b0 / a0;
    filter->b1 = b1 / a0;
    filter->b2 = b2 / a0;
    filter->a0 = 1.0f;
    filter->a1 = a1 / a0;
    filter->a2 = a2 / a0;
    filter->x1 = 0.0f;
    filter->x2 = 0.0f;
    filter->y1 = 0.0f;
    filter->y2 = 0.0f;
}

// Reset biquad filter state
static void biquad_reset(biquad_filter_t *filter) {
    filter->x1 = 0.0f;
    filter->x2 = 0.0f;
    filter->y1 = 0.0f;
    filter->y2 = 0.0f;
}

// Process single sample through biquad filter
static float biquad_process(biquad_filter_t *filter, float input) {
    // Direct Form II implementation
    float output = filter->b0 * input + filter->b1 * filter->x1 +
                   filter->b2 * filter->x2 - filter->a1 * filter->y1 -
                   filter->a2 * filter->y2;

    // Update delay line
    filter->x2 = filter->x1;
    filter->x1 = input;
    filter->y2 = filter->y1;
    filter->y1 = output;

    return output;
}

void audio_filters_init(audio_filters_t *filters, float sample_rate) {
    // High-pass filter coefficients (80 Hz cutoff for static/DC removal)
    float hp_freq = 320.0f;
    float hp_omega = 2.0f * PI * hp_freq / sample_rate;
    float hp_sin = sinf(hp_omega);
    float hp_cos = cosf(hp_omega);
    float hp_alpha = hp_sin / (2.0f * 0.707f); // Q = 0.707 (Butterworth)

    float hp_b0 = (1.0f + hp_cos) / 2.0f;
    float hp_b1 = -(1.0f + hp_cos);
    float hp_b2 = (1.0f + hp_cos) / 2.0f;
    float hp_a0 = 1.0f + hp_alpha;
    float hp_a1 = -2.0f * hp_cos;
    float hp_a2 = 1.0f - hp_alpha;

    // Low-pass filter coefficients (12 kHz cutoff for anti-aliasing/noise
    // reduction)
    float lp_freq = 3000.0f;
    float lp_omega = 2.0f * PI * lp_freq / sample_rate;
    float lp_sin = sinf(lp_omega);
    float lp_cos = cosf(lp_omega);
    float lp_alpha = lp_sin / (2.0f * 0.707f); // Q = 0.707 (Butterworth)

    float lp_b0 = (1.0f - lp_cos) / 2.0f;
    float lp_b1 = 1.0f - lp_cos;
    float lp_b2 = (1.0f - lp_cos) / 2.0f;
    float lp_a0 = 1.0f + lp_alpha;
    float lp_a1 = -2.0f * lp_cos;
    float lp_a2 = 1.0f - lp_alpha;

    // Initialize all filters
    biquad_init(&filters->hp_left, hp_b0, hp_b1, hp_b2, hp_a0, hp_a1, hp_a2);
    biquad_init(&filters->hp_right, hp_b0, hp_b1, hp_b2, hp_a0, hp_a1, hp_a2);
    biquad_init(&filters->lp_left, lp_b0, lp_b1, lp_b2, lp_a0, lp_a1, lp_a2);
    biquad_init(&filters->lp_right, lp_b0, lp_b1, lp_b2, lp_a0, lp_a1, lp_a2);
}

void audio_filters_process(audio_filters_t *filters, int16_t *buffer,
                           size_t samples) {
    for (size_t i = 0; i < samples; i += 2) {
        // Convert to float for processing
        float left = (float)buffer[i] / 32768.0f;
        float right = (float)buffer[i + 1] / 32768.0f;

        // Apply high-pass filter first (remove DC and low-frequency noise)
        left = biquad_process(&filters->hp_left, left);
        right = biquad_process(&filters->hp_right, right);

        // Apply low-pass filter second (remove high-frequency noise)
        left = biquad_process(&filters->lp_left, left);
        right = biquad_process(&filters->lp_right, right);

        // Convert back to int16_t with clamping
        float left_scaled = left * 32768.0f;
        float right_scaled = right * 32768.0f;

        // Clamp to prevent overflow
        if (left_scaled > 32767.0f)
            left_scaled = 32767.0f;
        if (left_scaled < -32768.0f)
            left_scaled = -32768.0f;
        if (right_scaled > 32767.0f)
            right_scaled = 32767.0f;
        if (right_scaled < -32768.0f)
            right_scaled = -32768.0f;

        buffer[i] = (int16_t)left_scaled;
        buffer[i + 1] = (int16_t)right_scaled;
    }
}

void audio_filters_reset(audio_filters_t *filters) {
    biquad_reset(&filters->hp_left);
    biquad_reset(&filters->hp_right);
    biquad_reset(&filters->lp_left);
    biquad_reset(&filters->lp_right);
}