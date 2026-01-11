#ifndef CODEC_FILTERS_H
#define CODEC_FILTERS_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Filter coefficients for high-pass filter (removes DC offset and low-frequency
// noise) Cutoff frequency: ~80 Hz at 44.1 kHz sample rate
typedef struct {
    float a0, a1, a2; // Denominator coefficients
    float b0, b1, b2; // Numerator coefficients
    float x1, x2;     // Previous input samples
    float y1, y2;     // Previous output samples
} biquad_filter_t;

typedef struct {
    biquad_filter_t hp_left;  // High-pass filter for left channel
    biquad_filter_t hp_right; // High-pass filter for right channel
    biquad_filter_t lp_left;  // Low-pass filter for left channel
    biquad_filter_t lp_right; // Low-pass filter for right channel
} audio_filters_t;

// Initialize filters for static removal
void audio_filters_init(audio_filters_t *filters, float sample_rate);

// Process stereo audio buffer with filters
void audio_filters_process(audio_filters_t *filters, int16_t *buffer,
                           size_t samples);

// Reset filter states (useful when changing sample rates)
void audio_filters_reset(audio_filters_t *filters);

#endif