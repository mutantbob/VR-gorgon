precision highp float;

varying vec3 ray;
uniform float phase;

void main() {

    vec3 rayn = normalize(ray);

    bool a = 0.5 <= mod(rayn.x*10.0 + phase*5.0, 1.0);
    bool b = 0.5 <= mod(rayn.y*10.0 + phase*4.0, 1.0);
    bool c = 0.5 <= mod(rayn.z*10.0 + phase*3.0, 1.0);

    float g;
    if (a^^b^^c) {
        g = 1.0;
    } else {
        g = 0.0;
    }

    gl_FragColor = vec4(g,g,g, 1.0);
}
