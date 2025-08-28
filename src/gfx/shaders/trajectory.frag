#version 330 core

uniform vec4 surface_color;
uniform float curr_ecc_anom;
uniform float anomaly_range;
uniform float eccentricity;

in float v_ecc_anom;

const float MIN_ALPHA = 0.18;
const float DIFF_MULTIPLIER = 1.0 - MIN_ALPHA;

layout (location = 0) out vec4 outColor;

float angle_diff(float a, float b) {
    float d = a - b;
    if (eccentricity < 1.0) {
        d = mod(d, anomaly_range);
    } else {
        d = 1.0 - d;
    }
    return d;
}

float get_alpha(float v_ecc_anom, float curr_ecc_anom) {
    if (eccentricity >= 1.0) {
        v_ecc_anom = -v_ecc_anom;
        if (v_ecc_anom > curr_ecc_anom) {
            return MIN_ALPHA;
        }

        float behind_amount = curr_ecc_anom - v_ecc_anom;
        float behind_frac = behind_amount / (0.5 * anomaly_range);

        return max(DIFF_MULTIPLIER * (1.0 - behind_frac) + MIN_ALPHA, MIN_ALPHA);
    }

    float diff = angle_diff(v_ecc_anom, curr_ecc_anom);

    return max(DIFF_MULTIPLIER * diff / anomaly_range + MIN_ALPHA, MIN_ALPHA);
}

// Dropoff alpha as distances become extreme and
// floats become imprecise
float extreme_alpha_dropoff(float v_ecc_anom) {
    float ecc_anom = abs(v_ecc_anom);

    if (ecc_anom < 9.0) {
        return 1.0;
    }

    return 10.0 - ecc_anom;
}

void main()
{
    outColor = surface_color;

    outColor.a *= get_alpha(v_ecc_anom, curr_ecc_anom);
    outColor.a *= extreme_alpha_dropoff(v_ecc_anom);

    // the definition of color_mapping is external
    // and added at runtime; ignore the error
    outColor.rgb = color_mapping(outColor.rgb);
}