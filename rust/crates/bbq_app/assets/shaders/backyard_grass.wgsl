#import bevy_pbr::{
    decal::clustered::apply_decals,
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Fade against the lawn underneath each blade, including on worn patches and in shade.
    // World-space distance keeps the response steady when turning in place and across tile joins.
    let range = distance(in.world_position.xz, view.world_position.xz);
    if range >= 23.0 {
        discard;
    }
    let visibility = 1.0 - smoothstep(7.0, 23.0, range);
    var input = pbr_input_from_standard_material(in, is_front);
    input.material.base_color.a *= visibility;
    apply_decals(&input);
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(input, apply_pbr_lighting(input));
    return out;
}
