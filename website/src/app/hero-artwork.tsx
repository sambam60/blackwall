"use client";

import { Dithering, GrainGradient } from "@paper-design/shaders-react";

export function HeroArtwork() {
  return (
    <div className="absolute top-0 left-0 bottom-0 w-[65%] max-lg:hidden overflow-hidden">
      <div className="absolute inset-0 bg-black" />
      <img
        src="/hero-art.png"
        alt=""
        aria-hidden="true"
        className="absolute inset-0 w-full h-full object-cover blur-[9px] scale-105"
      />
      <GrainGradient
        speed={1}
        scale={0.58}
        rotation={0}
        offsetX={0.06}
        offsetY={0}
        softness={0.49}
        intensity={0.7}
        noise={0.21}
        shape="wave"
        colors={["#08090A", "#1B0504", "#2D070C", "#930D0B", "#9D3444"]}
        colorBack="#00000000"
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          opacity: 0.6,
          backgroundColor: "#000000",
        }}
      />
      <Dithering
        speed={1}
        shape="simplex"
        type="4x4"
        size={2.4}
        scale={0.38}
        frame={1080151.8}
        colorBack="#00000000"
        colorFront="#FFFFFF"
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          mixBlendMode: "color-burn",
          backgroundColor: "#000000",
        }}
      />
      <div className="absolute inset-y-0 right-0 w-2/5 bg-gradient-to-r from-transparent to-[#050505] backdrop-blur-xl" />
    </div>
  );
}
