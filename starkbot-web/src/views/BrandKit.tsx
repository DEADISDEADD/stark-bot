import { Download } from 'lucide-react'
import { Navbar } from '../components/Navbar'
import { Footer } from '../components/Footer'
import { Stars } from '../components/Stars'
import { GridBackground } from '../components/GridBackground'

function LogoCard({ src, name }: { src: string; name: string }) {
  return (
    <div className="flex flex-col items-center gap-6 p-8 bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 hover:border-blue-500/30 transition-all duration-300">
      <div className="w-full flex items-center justify-center p-6 bg-[#111] rounded-xl min-h-[200px]">
        <img src={src} alt={name} className="max-h-40 object-contain" />
      </div>
      <div className="flex flex-col items-center gap-3 w-full">
        <span className="text-white/70 text-sm font-medium">{name}</span>
        <a
          href={src}
          download
          className="flex items-center gap-2 px-4 py-2 bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 rounded-lg transition-all duration-300 border border-blue-500/20 hover:border-blue-500/40 text-sm"
        >
          <Download className="w-4 h-4" />
          Download PNG
        </a>
      </div>
    </div>
  )
}

function ColorSwatch({ color, name, hex }: { color: string; name: string; hex: string }) {
  return (
    <div className="flex flex-col gap-2">
      <div
        className="w-full h-20 rounded-xl border border-white/10"
        style={{ backgroundColor: hex }}
      />
      <span className="text-white text-sm font-medium">{name}</span>
      <span className="text-white/50 text-xs font-mono">{hex}</span>
    </div>
  )
}

export default function BrandKit() {
  return (
    <div className="min-h-screen overflow-x-hidden">
      <Stars />
      <GridBackground />
      <div className="relative z-10">
        <Navbar />

        <main className="pt-32 pb-20 px-6">
          <div className="max-w-4xl mx-auto">
            {/* Header */}
            <div className="text-center mb-16">
              <h1 className="text-4xl md:text-5xl font-bold mb-4">
                Brand Kit
              </h1>
              <p className="text-white/60 text-lg max-w-2xl mx-auto">
                Official logos and brand assets for StarkBot. Use these when referencing or integrating with StarkBot.
              </p>
            </div>

            {/* Logos */}
            <section className="mb-20">
              <h2 className="text-2xl font-semibold mb-2">Logos</h2>
              <p className="text-white/50 mb-8">Download the official StarkBot logos for use in your projects and integrations.</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <LogoCard src="/starkbot.png" name="StarkBot Logo" />
                <LogoCard src="/starkbot2.png" name="StarkBot Logo (Alt)" />
              </div>
            </section>

            {/* Colors */}
            <section className="mb-20">
              <h2 className="text-2xl font-semibold mb-2">Colors</h2>
              <p className="text-white/50 mb-8">The core color palette used across StarkBot.</p>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-4">
                <ColorSwatch name="Background" hex="#1a1a1a" color="bg-[#1a1a1a]" />
                <ColorSwatch name="Blue 500" hex="#3b82f6" color="bg-blue-500" />
                <ColorSwatch name="Blue 700" hex="#1d4ed8" color="bg-blue-700" />
                <ColorSwatch name="Blue 400" hex="#60a5fa" color="bg-blue-400" />
                <ColorSwatch name="White" hex="#ffffff" color="bg-white" />
              </div>
            </section>

            {/* Usage Guidelines */}
            <section>
              <h2 className="text-2xl font-semibold mb-2">Usage Guidelines</h2>
              <p className="text-white/50 mb-8">Please keep these in mind when using our brand assets.</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="p-6 bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10">
                  <h3 className="text-green-400 font-semibold mb-3">Do</h3>
                  <ul className="text-white/60 text-sm space-y-2">
                    <li>Use the logo on a dark background for best contrast</li>
                    <li>Maintain the original aspect ratio</li>
                    <li>Leave sufficient clear space around the logo</li>
                  </ul>
                </div>
                <div className="p-6 bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10">
                  <h3 className="text-red-400 font-semibold mb-3">Don't</h3>
                  <ul className="text-white/60 text-sm space-y-2">
                    <li>Distort, rotate, or alter the logo colors</li>
                    <li>Place the logo on busy or low-contrast backgrounds</li>
                    <li>Use the logo to imply endorsement without permission</li>
                  </ul>
                </div>
              </div>
            </section>
          </div>
        </main>

        <Footer />
      </div>
    </div>
  )
}
