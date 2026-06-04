import { ImageResponse } from "next/og";

export const runtime = "edge";
export const alt = "InheritX - Secure Wealth Inheritance & Asset Planning";
export const size = {
  width: 1200,
  height: 630,
};
export const contentType = "image/png";

export default async function Image() {
  return new ImageResponse(
    (
      <div
        style={{
          background: "linear-gradient(135deg, #161E22 0%, #1C252A 100%)",
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "80px",
          position: "relative",
        }}
      >
        {/* Background pattern */}
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            opacity: 0.1,
            background:
              "radial-gradient(circle at 20% 50%, #33C5E0 0%, transparent 50%), radial-gradient(circle at 80% 50%, #33C5E0 0%, transparent 50%)",
          }}
        />

        {/* Content */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            textAlign: "center",
            zIndex: 1,
          }}
        >
          <h1
            style={{
              fontSize: "72px",
              fontWeight: "bold",
              color: "#FCFFFF",
              marginBottom: "24px",
              lineHeight: 1.2,
            }}
          >
            InheritX
          </h1>
          <p
            style={{
              fontSize: "36px",
              color: "#92A5A8",
              marginBottom: "40px",
              maxWidth: "900px",
              lineHeight: 1.4,
            }}
          >
            Secure Wealth Inheritance & Asset Planning
          </p>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "16px",
              padding: "20px 40px",
              background: "#33C5E0",
              borderRadius: "12px",
              color: "#000",
              fontSize: "28px",
              fontWeight: "600",
            }}
          >
            From your hands, to theirs — without a hitch
          </div>
        </div>

        {/* Footer */}
        <div
          style={{
            position: "absolute",
            bottom: "40px",
            display: "flex",
            alignItems: "center",
            gap: "12px",
            color: "#92A5A8",
            fontSize: "24px",
          }}
        >
          <span>inheritx.com</span>
        </div>
      </div>
    ),
    {
      ...size,
    }
  );
}
