MC3479
NSCDJJN005NDUNV

# Filters
https://www.allaboutcircuits.com/video-tutorials/op-amps-low-pass-and-high-pass-active-filters/
http://sim.okawa-denshi.jp/en/OPstool.php
https://tools.analog.com/en/filterwizard/

https://electronics.stackexchange.com/questions/55233/how-do-you-simulate-voltage-noise-with-ltspice
https://electronics.stackexchange.com/questions/291721/how-to-implement-frequency-sweep-in-transient-mode-in-ltspice

## LTSpice

`.step param freq list 0.01 0.1 1 2 4 8 10 12` en dan BV voltage source (misc components) met `V=0.015*sin(2*pi*freq*time)`

