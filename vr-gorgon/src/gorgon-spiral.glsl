precision highp float;

varying vec3 ray;
uniform float phase;

#define PI 3.1415926538

void main()
{
    vec3 rayn = normalize(ray);

    float r = length(rayn.xy);

    float theta = atan(r, rayn.z);
    float phi = atan(rayn.y, rayn.x);

    bool a = 0.5 > mod(phi*4.0/PI + theta + phase*4.0, 1.0);
    bool b = 0.5 > mod( theta * 20.0 / PI + sin(mod(phase*6.0, 2.0)*PI), 1.0);

    float g;
    if (a ^^ b) {
	g = 1.0;
    } else {
	g = 0.0;
    }
    gl_FragColor = vec4(g,g,g, 1.0);
}
