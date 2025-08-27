#version 330 core

// TERMINOLOGY:
// "Point" refers to a single point in the orbit.
// This isn't the same as "vertex", which is a single
// point in the mesh.

uniform mat4 u_proj_view;       // Projection * view
uniform mat4 u_tf;              // pqw, rp=1 orbit → render-world space
uniform float u_eccentricity;   // eccentricity
uniform float u_a_norm;         // semi-major axis if rp=1
uniform float u_b_norm;         // semi-minor axis if rp=1
uniform float u_start_ecc_anom; // starting hyp. ecc. anom.
uniform uint u_vertex_count;    // number of vertices
uniform float u_thickness_px;
uniform vec2 u_viewport;
uniform float u_ecc_anom_range; // ecc. anom. range size

out float v_ecc_anom;

float get_eccentric_anomaly(int idx) {
    if (u_eccentricity < 1.0) {
        return float(idx) / float(u_vertex_count) * u_ecc_anom_range;
    } else {
        return -float(idx) / float(u_vertex_count) * u_ecc_anom_range - u_start_ecc_anom;
    }
}

vec3 get_point_at_eccentric_anomaly(float ecc_anom) {
    if (u_eccentricity < 1.0) {
        float cos_ecc_anom = cos(ecc_anom);
        float sin_ecc_anom = sin(ecc_anom);
        return vec3(
            u_a_norm * (cos_ecc_anom - u_eccentricity),
            u_b_norm * sin_ecc_anom,
            0.0
        );
    } else {
        float cosh_ecc_anom = cosh(ecc_anom);
        float sinh_ecc_anom = sinh(ecc_anom);
        return vec3(
            u_a_norm * (cosh_ecc_anom - u_eccentricity),
            u_b_norm * sinh_ecc_anom,
            0.0
        );
    }
}

void main() {
    int vertex_idx = gl_VertexID;
    int curr_point_idx = vertex_idx / 2;
    float side = (vertex_idx % 2 == 0) ? -1.0 : 1.0;
    
    // We don't care about overflow/underflow.
    // We're not indexing an array, we're putting
    // it in a math equation.
    int next_point_idx = curr_point_idx + 1;

    float curr_ecc_anom = get_eccentric_anomaly(curr_point_idx);
    float next_ecc_anom = get_eccentric_anomaly(next_point_idx);

    v_ecc_anom = curr_ecc_anom;

    vec3 curr_pqw = get_point_at_eccentric_anomaly(curr_ecc_anom);
    vec3 next_pqw = get_point_at_eccentric_anomaly(next_ecc_anom);

    vec4 curr_clip = u_proj_view * (u_tf * vec4(curr_pqw, 1.0));
    vec4 next_clip = u_proj_view * (u_tf * vec4(next_pqw, 1.0));

    float aspect = u_viewport.x / u_viewport.y;

    float eps = 1e-6;
    vec2 curr_ndc = aspect * curr_clip.xy / max(curr_clip.w, eps);
    vec2 next_ndc = aspect * next_clip.xy / max(next_clip.w, eps);

    vec2 ndc_diff = next_ndc - curr_ndc;
    vec2 dir = (length(ndc_diff) < 1e-4)
        ? vec2(0.0, 1.0)
        : normalize(ndc_diff);
    vec2 normal = vec2(-dir.y, dir.x);

    float ndc_per_pixel = (u_viewport.y > 0.0) ? (2.0 / u_viewport.y) : 0.0;
    vec2 offset_ndc = normal * (u_thickness_px * 0.5 * ndc_per_pixel);

    offset_ndc.x /= aspect;

    vec4 offset_clip = vec4(
        offset_ndc * curr_clip.w * side,
        0.0,
        0.0
    );

    gl_Position = curr_clip + offset_clip;
}