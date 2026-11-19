"""Synthesises fpvdrone's motor loop: four slightly detuned motors (their beating is the
quad's 'wobble'), blade-pass harmonics, and band-limited prop wash noise. Every component
has a whole number of cycles in the loop, so it repeats seamlessly. The game plays it at
several pitches and crossfades them by motor speed."""
import sys
import numpy as np
import soundfile as sf

RATE = 44100
SECONDS = 2.0
N = int(RATE * SECONDS)
t = np.arange(N) / RATE
rng = np.random.default_rng(7)

def tone(freq):
    # Snap to a whole number of cycles over the loop.
    f = round(freq * SECONDS) / SECONDS
    return f

out = np.zeros(N)
# Four motors around 180 Hz (blade-pass at mid throttle), detuned for beating.
for base in (178.0, 180.5, 182.0, 184.5):
    for h, amp in ((1, 1.0), (2, 0.55), (3, 0.35), (4, 0.22), (5, 0.12), (6, 0.08), (8, 0.05)):
        f = tone(base * h)
        phase = rng.uniform(0, 2 * np.pi)
        out += amp * np.sin(2 * np.pi * f * t + phase)
out /= 4.0

# Prop wash: noise shaped to 300-4000 Hz, periodic over the loop (built in the frequency domain).
spec = np.zeros(N // 2 + 1, dtype=complex)
freqs = np.fft.rfftfreq(N, 1 / RATE)
band = (freqs > 300) & (freqs < 4000)
shape = np.where(band, 1.0 / np.sqrt(np.maximum(freqs, 1.0) / 300.0), 0.0)
spec[band] = shape[band] * np.exp(1j * rng.uniform(0, 2 * np.pi, band.sum()))
noise = np.fft.irfft(spec, n=N)
noise /= np.abs(noise).max()
out = out / np.abs(out).max() + 0.25 * noise

out = 0.8 * out / np.abs(out).max()
sf.write(sys.argv[1], out.astype(np.float32), RATE, format="OGG", subtype="VORBIS")
print("wrote", sys.argv[1], f"{SECONDS}s")
