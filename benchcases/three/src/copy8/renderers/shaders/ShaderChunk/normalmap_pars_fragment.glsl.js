export default /* glsl */`
#ifdef USE_NORMALMAP

	uniform sampler2D normalMap;
	uniform vec2 normalScale;

#endif

#ifdef OBJECTSPACE_NORMALMAP

	uniform mat3 normalMatrix;

#endif

#if ! defined ( USE_TANGENT ) && ( defined ( TANGENTSPACE_NORMALMAP ) || defined ( USE_CLEARCOAT_NORMALMAP ) )

	// Per-Pixel Tangent Space Normal Mapping
	// http://hacksoflife.blogspot.ch/2009/11/per-pixel-tangent-space-normal-mapping.html

	vec3 perturbNormal2Arb( vec3 eye_pos, vec3 surf_norm, vec2 normalScale, in sampler2D normalMap ) {

		// Workaround for Adreno 3XX dFd*( vec3 ) bug. See #9988

		vec3 q0 = vec3( dFdx( eye_pos.x ), dFdx( eye_pos.y ), dFdx( eye_pos.z ) );
		vec3 q1 = vec3( dFdy( eye_pos.x ), dFdy( eye_pos.y ), dFdy( eye_pos.z ) );
		vec2 st0 = dFdx( vUv.st );
		vec2 st1 = dFdy( vUv.st );

		float scale = sign( st1.t * st0.s - st0.t * st1.s ); // we do not care about the magnitude

		vec3 S = normalize( ( q0 * st1.t - q1 * st0.t ) * scale );
		vec3 T = normalize( ( - q0 * st1.s + q1 * st0.s ) * scale );
		vec3 N = normalize( surf_norm );

		vec3 mapN = texture2D( normalMap, vUv ).xyz * 2.0 - 1.0;

		mapN.xy *= normalScale;

		#ifdef DOUBLE_SIDED

			// Workaround for Adreno GPUs gl_FrontFacing bug. See #15850 and #10331
			// http://hacksoflife.blogspot.com/2009/11/per-pixel-tangent-space-normal-mapping.html?showComment=1522254677437#c5087545147696715943
			vec3 NfromST = cross( S, T );
			if( dot( NfromST, N ) > 0.0 ) {

				S *= -1.0;
				T *= -1.0;

			}

		#else

			mapN.xy *= ( float( gl_FrontFacing ) * 2.0 - 1.0 );

		#endif

		mat3 tsn = mat3( S, T, N );
		return normalize( tsn * mapN );

	}

#endif
`;
