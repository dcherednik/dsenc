# dsenc
Experimental dsd compatible encoder

It is just experimental project to make some fun with 1-bit delta-sigma modulation.
It can be useful to encode DSD64 from 44100 pcm, but the quality will be far from perfect
due to simplest second order delta sigma modulator.

Use FFT/IFFT to 64x upsample, so now it is slowest part of encoder.

