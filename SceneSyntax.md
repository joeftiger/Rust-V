# Scene Syntax
The scene file is written in the `RON` format and consists of following main
blocks:

- `config`: Basic configuration regarding filename, number of passes, etc.
- `camera`: The camera setup for the scene.
- `integrator`: The integrator to use (Whitted, Path, Spectral, ...).
- `sampler`: The random sampler to use.
- `scene`: Contains all the scene objects

Basically the syntax looks like the following:
```ron
(
	config: (...),
	camera: {...}, // notice curly brackets
	integrator: {...},
	sampler: ...,
	scene: (...),
)
```

Due to code internal representation it is important to notice the usage of both
`()` and `{}`.

## Examples
Look into `$PROJECT_ROOT/scenes/`. There are some exaple scenes.

## Short Forms
Throughout the file we'll reference elements like vectors in a short form which
use the following syntax:

### `Vec2`
```ron
Vec2 ( // name may be omitted
	x: float,
	y: float,
)
```

### `Vec3`
```ron
Vec3 ( // name may be omitted
	x: float,
	y: float,
	z: float,
)
```

### `UBounds2`
```ron
UBounds2 ( // name may be omitted
	min: Vec2,
	max: Vec2,		// not checked to really be max!
)
```

### `Option<T>`
Same as in rust code:
```ron
Some( T )
// or
None
```

### `SerdeColors`
Describes colors and has some nice shortcuts:

#### `Srgb`
May convert to `Xyz` if needed.
```ron
Srgb (
	[ r, g, b ],
)
```

#### `Xyz`
May convert to `Srgb` if needed.
```ron
Xyz (
	[ x, y, z ],
)
```

#### `Spectrum`
May convert to `Srgb` or `Xyz` if needed.
```ron
Spectrum (
	[ ... ], // 36 entries spanning 380nm to 730 nm in 10nm steps
)
```

#### `Color`
Contains pre-defined colors for `Srgb, Xyz, Spectrum`.
```ron
Color( $c )
```
where `$c` is one of the following:
- DarkSkin
- LightSkin
- BlueSky
- Foliage
- BlueFlower
- BluishGreen
- Orange
- PurplishBlue
- ModerateRed
- Purple,
- YellowGreen
- OrangeYellow
- Blue
- Green
- Red
- Yellow
- Magenta
- Cyan
- White
- Grey1
- Grey2
- Grey3
- Grey4
- Black

#### `MulColor`
Scales a pre-defined color.
```ron
MulColor( float, $c )
```

#### `Constant`
A constant color.
```ron
Constant( float )
```

### `Sampler`
The sampler for random numbers. A bit useless if not taking `Random`.
```ron
Random
// or
NoOp // always 0.5
```

### `CameraSampler`
The sampling method for pixels in the camera.
```ron
NoOp	// always Vec2 (x: 0.5, y: 0.5)
// or
Random
// or
Concentric
// or
NonConcentric
```

### `SpectralSampler`
The spectral sampling method for the spectral path tracer.
```ron
Random
// or
Hero
```

### `DirectLightStrategy`
Defines how we want to calculate the direct illumination.
```ron
All         // calculates influence of all lights
// or
Random      // calculates influence of only one random light
```


## `config`

The basic configuration contains following key-value pairs:
```
config: (
	filename: Option<String>,	// output image
	bounds: Option<Vec2>,		// only trace a part of the whole resolution
	block_size: Vec2,			// threads trace pixels in bulk
	passes: u32,				// number of passes per pixel
	threads: Option<u32>,		// Defaults to all available threads
)
```

## `camera`
Currently only one camera works:

```ron
camera: {
	"PerspectiveCamera": (
		Sampler: CameraSampler,	
		Position: Vec3,			// where is the camera
		Target: Vec3,			// where does it look at
		Up: Vec3,				// needed to orient the camera correctly
		FovY: float,			// the field of view in Y axis.
		Resolution: Vec2,		// the resolution of the image basically
	)
}
```

## `integrator`
Describes the method how the pixel color is calculated.
```ron
integrator: {
	$method
}
```
The available methods are described below:

### Whitted
The classic Whitted method.
```ron
"Whitted": (
	max_depth: u32,				// depth of light bounces
)
```

### Path tracing
Path tracing takes into account direct illumination at each light bounce.
```ron
"Path": (
	max_depth: u32,				// depth of light bounces
)
```

### Spectral path tracing
Traces lights in bundles until specular materials split wavelengths into
different directions, tracing each separatly afterwards.
```ron
"SpectralPath": (
	max_depth: u32,				// depth of light bounces
	light_wave_samples: u32,	// number of wavelengths to follow in bulk
	spectral_sampler: SpectralSampler,
)
```


## `sampler`
The random number generator. Basically only `Random` makes sense.
```ron
Random							// random numbers in [0, 1)
// or
NoOp							// 0.5
```

## `scene`
The heart describing the objects inside the scene. The basic syntax is:
```ron
scene: (
	bounding_box: Aabb,			// a geometry, described below
    objects: [ ... ],			// array of objects
)
```

### `objects`
Each object is either a **receiver** or an **emitter**.
They have the following syntax:
```ron
Receiver((
	geometry: { $geom },
	bsdf: (						// bidirectional scattering distribution functions
		bxdfs: [ ... ]			// list of BxDFs
	),
))

// or

Emitter((
	geometry: { ... },
	bsdf: { ... },
	emission: SerdeColor,
))
```

#### Geometries
There are various geometries available, each one described below.

##### Point
```ron
"Point": ( Vec3 )
```

##### Plane
```ron
"Plane": (
	normal: Vec3,
	d: float,					// the offset into the normal direction
)
```

##### Disk
```ron
"Disk": (
	center: Vec3,
	normal: Vec3,
	radius: float,
)
```

##### Aabb
The axis-aligned bounding box is a cube that is aligned to the x-y-z axis.
```ron
"Aabb": (
	min: Vec3,
	max: Vec3,
)
```

##### Sphere
```ron
"Sphere": (
	center: Vec3,
	radius: float,
)
```

##### Cylinder
```ron
"Cylinder": (
	caps: (Vec3, Vec3),
	radius: float,
)
```

##### Bubble
A sphere inside a sphere
```ron
"Bubble": (
	inner: Sphere,				// should be INSIDE the outer sphere! Unchecked
	outer: Sphere,
)
```

##### Biconvex lens
```ron
"BiconvexLens": (
	sphere0: Sphere,
	sphere1: Sphere,
)
```

##### Mesh
```ron
"Mesh": (
	Vertices: [ Vec3, Vec3, ... ],
	VertexNormals: [ Vec3, Vec3, ... ],		// optional, may be omitted
	Faces: [ $face, $face, ... ],
	Bounds: Aabb,
	ShadingMode: $shading,
)
```
The `$shading` mode is either `Flat` or `Phong` (simple interpolation).

The faces are described by 3 vertex indices and possible vertex normals:
```
Face ( // struct name may be omitted
	v: (usize, usize, usize),				// indices of Vertices
	vn: Option<(usize, usize, usize)>,		// indices of VertexNormals
)
```

#### BSDF
The bidirectional scattering distribution function is a list of 
bidirectional reflecting/transmitting distribution functions that get chosen at
random.

The list contains of of the following BxDFS:

##### Lambertian
```ron
"LambertianReflection": (
	r: SerdeColor,
)

// or

"LambertianTransmission": (
	t: SerdeColor,
)
```

##### Oren Nayar
A more natural looking diffuse surface.
```ron
"OrenNayar": (
    r: ColorSerde,
    a: float,					// parameter A
    b: float,					// parameter B
)
```

##### Specular
```ron
"SpecularReflection": (
    r: ColorSerde,
    fresnel: FresnelType,		// described below
)

// or

"SpecularTransmission": (
    t: ColorSerde,
    fresnel: FresnelDielectric,
)

// or

// a combination of reflection and transmission
"FresnelSpecular": (
    r: ColorSerde,
    t: ColorSerde,
    fresnel: FresnelDielectric,
)
```
A fresnel type is one of the following choices:
- `Dielectric(FresnelDielectric)`: fresnel implementation for dielectric
								   materials.
- `NoOp`: A no-operation Fresnel implementation that returns 100% reflection for
		  all incoming directions. Although this is physically implausible, it
		  is a convenient capability to have available.

The `FresnelDielectric` is described by
```ron
FresnelDielectric (	// struct name may be omitted
	eta_i: RefractiveType,
	eta_t: RefractiveType,
)
```

There exist several refractive types. Choose one of the following:
- Air
- Vacuum
- Water
- Glass
- Sapphire
- Linear2
- Linear4
- Linear6
- Linear8
- Linear10
The linear types simply map the wavelength linearly between `[1, num]`.

