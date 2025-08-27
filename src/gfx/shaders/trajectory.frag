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
    d = mod(d + anomaly_range / 2.0, anomaly_range);
    return d;
}

float get_alpha(float v_ecc_anom, float curr_ecc_anom) {
    if (eccentricity > 1.0 && v_ecc_anom > curr_ecc_anom) {
        return MIN_ALPHA;
    }

    float diff = angle_diff(v_ecc_anom, curr_ecc_anom);

    return max(DIFF_MULTIPLIER * diff / anomaly_range + MIN_ALPHA, MIN_ALPHA);
}

void main()
{
    outColor = surface_color;

    outColor.a *= get_alpha(v_ecc_anom, curr_ecc_anom);

    // the definition of color_mapping is external
    // and added at runtime; ignore the error
    outColor.rgb = color_mapping(outColor.rgb);
}