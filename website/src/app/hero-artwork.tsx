"use client";

import { Dithering, GrainGradient } from "@paper-design/shaders-react";
import { useTheme } from "./theme-provider";

export function HeroArtwork() {
  const { theme } = useTheme();
  const dark = theme === "dark";

  return (
    <div className="absolute top-0 left-0 bottom-0 w-[65%] max-lg:hidden overflow-hidden">
      <div
        className="absolute inset-0"
        style={{ backgroundColor: dark ? "#000000" : "#ffffff" }}
      />
      <img
        src={dark ? "/hero-art.png" : "/hero-art-light.jpg"}
        alt=""
        aria-hidden="true"
        className="absolute inset-0 w-full h-full object-cover blur-[9px] scale-105"
        style={{ opacity: dark ? 1 : 0.1 }}
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
        colors={
          dark
            ? ["#08090A", "#1B0504", "#2D070C", "#930D0B", "#9D3444"]
            : ["#f0f4ff", "#d4e0ff", "#8fb5ff", "#3b72f6", "#1d4ed8"]
        }
        colorBack="#00000000"
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          opacity: 0.6,
          backgroundColor: dark ? "#000000" : "#ffffff",
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
        colorFront={dark ? "#FFFFFF" : "#3b82f6"}
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          opacity: dark ? 1 : 0.12,
          mixBlendMode: dark ? "color-burn" : "multiply",
          backgroundColor: dark ? "#000000" : "#ffffff",
        }}
      />
      <div
        className="absolute inset-y-0 right-0 w-2/5 backdrop-blur-sm"
        style={{
          background: dark
            ? "linear-gradient(to right, transparent, #050505)"
            : "linear-gradient(to right, transparent, #ffffff)",
        }}
      />
    </div>
  );
}
