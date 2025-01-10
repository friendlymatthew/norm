3.1     Basic format test files (non-interlaced)
------------------------------------------------

        basn0g01    -   black & white
        basn0g02    -   2 bit (4 level) grayscale
        basn0g04    -   4 bit (16 level) grayscale
        basn0g08    -   8 bit (256 level) grayscale
        basn0g16    -   16 bit (64k level) grayscale
        basn2c08    -   3x8 bits rgb color
        basn2c16    -   3x16 bits rgb color
        basn3p01    -   1 bit (2 color) paletted
        basn3p02    -   2 bit (4 color) paletted
        basn3p04    -   4 bit (16 color) paletted
        basn3p08    -   8 bit (256 color) paletted
        basn4a08    -   8 bit grayscale + 8 bit alpha-channel
        basn4a16    -   16 bit grayscale + 16 bit alpha-channel
        basn6a08    -   3x8 bits rgb color + 8 bit alpha-channel
        basn6a16    -   3x16 bits rgb color + 16 bit alpha-channel



3.2     Basic format test files (Adam-7 interlaced)
---------------------------------------------------

        basi0g01    -   black & white
        basi0g02    -   2 bit (4 level) grayscale
        basi0g04    -   4 bit (16 level) grayscale
        basi0g08    -   8 bit (256 level) grayscale
        basi0g16    -   16 bit (64k level) grayscale
        basi2c08    -   3x8 bits rgb color
        basi2c16    -   3x16 bits rgb color
        basi3p01    -   1 bit (2 color) paletted
        basi3p02    -   2 bit (4 color) paletted
        basi3p04    -   4 bit (16 color) paletted
        basi3p08    -   8 bit (256 color) paletted
        basi4a08    -   8 bit grayscale + 8 bit alpha-channel
        basi4a16    -   16 bit grayscale + 16 bit alpha-channel
        basi6a08    -   3x8 bits rgb color + 8 bit alpha-channel
        basi6a16    -   3x16 bits rgb color + 16 bit alpha-channel



3.3     Size test files
-----------------------

        s01n3p01    -   1x1 paletted file, no interlacing
        s02n3p01    -   2x2 paletted file, no interlacing
        s03n3p01    -   3x3 paletted file, no interlacing
        s04n3p01    -   4x4 paletted file, no interlacing
        s05n3p02    -   5x5 paletted file, no interlacing
        s06n3p02    -   6x6 paletted file, no interlacing
        s07n3p02    -   7x7 paletted file, no interlacing
        s08n3p02    -   8x8 paletted file, no interlacing
        s09n3p02    -   9x9 paletted file, no interlacing
        s32n3p04    -   32x32 paletted file, no interlacing
        s33n3p04    -   33x33 paletted file, no interlacing
        s34n3p04    -   34x34 paletted file, no interlacing
        s35n3p04    -   35x35 paletted file, no interlacing
        s36n3p04    -   36x36 paletted file, no interlacing
        s37n3p04    -   37x37 paletted file, no interlacing
        s38n3p04    -   38x38 paletted file, no interlacing
        s39n3p04    -   39x39 paletted file, no interlacing
        s40n3p04    -   40x40 paletted file, no interlacing

        s01i3p01    -   1x1 paletted file, interlaced
        s02i3p01    -   2x2 paletted file, interlaced
        s03i3p01    -   3x3 paletted file, interlaced
        s04i3p01    -   4x4 paletted file, interlaced
        s05i3p02    -   5x5 paletted file, interlaced
        s06i3p02    -   6x6 paletted file, interlaced
        s07i3p02    -   7x7 paletted file, interlaced
        s08i3p02    -   8x8 paletted file, interlaced
        s09i3p02    -   9x9 paletted file, interlaced
        s32i3p04    -   32x32 paletted file, interlaced
        s33i3p04    -   33x33 paletted file, interlaced
        s34i3p04    -   34x34 paletted file, interlaced
        s35i3p04    -   35x35 paletted file, interlaced
        s36i3p04    -   36x36 paletted file, interlaced
        s37i3p04    -   37x37 paletted file, interlaced
        s38i3p04    -   38x38 paletted file, interlaced
        s39i3p04    -   39x39 paletted file, interlaced
        s40i3p04    -   40x40 paletted file, interlaced



3.4     Alpha-channel (and background) test files
-------------------------------------------------

        bgbn4a08    -   8 bit grayscale, alpha, black background chunk
        bggn4a16    -   16 bit grayscale, alpha, gray background chunk
        bgwn6a08    -   3x8 bits rgb color, alpha, white background chunk
        bgyn6a16    -   3x16 bits rgb color, alpha, yellow background chunk

        bgai4a08    -   8 bit grayscale, alpha, no background chunk
        bgai4a16    -   16 bit grayscale, alpha, no background chunk
        bgan6a08    -   3x8 bits rgb color, alpha, no background chunk
        bgan6a16    -   3x16 bits rgb color, alpha, no background chunk



3.5     Transparency (and background) test files
------------------------------------------------

        tp0n1g08    -   not transparent for reference (logo on gray)
        tbbn1g04    -   transparent, black background chunk
        tbwn1g16    -   transparent, white background chunk
        tp0n2c08    -   not transparent for reference (logo on gray)
        tbrn2c08    -   transparent, red background chunk
        tbgn2c16    -   transparent, green background chunk
        tbbn2c16    -   transparent, blue background chunk
        tp0n3p08    -   not transparent for reference (logo on gray)
        tp1n3p08    -   transparent, but no background chunk
        tbbn3p08    -   transparent, black background chunk
        tbgn3p08    -   transparent, light-gray background chunk 
        tbwn3p08    -   transparent, white background chunk
        tbyn3p08    -   transparent, yellow background chunk



3.6     Gamma test files
------------------------

        g03n0g16    -   grayscale, file-gamma = 0.35
        g04n0g16    -   grayscale, file-gamma = 0.45
        g05n0g16    -   grayscale, file-gamma = 0.55
        g07n0g16    -   grayscale, file-gamma = 0.70
        g10n0g16    -   grayscale, file-gamma = 1.00
        g25n0g16    -   grayscale, file-gamma = 2.50
        g03n2c08    -   color, file-gamma = 0.35
        g04n2c08    -   color, file-gamma = 0.45
        g05n2c08    -   color, file-gamma = 0.55
        g07n2c08    -   color, file-gamma = 0.70
        g10n2c08    -   color, file-gamma = 1.00
        g25n2c08    -   color, file-gamma = 2.50
        g03n3p04    -   paletted, file-gamma = 0.35
        g04n3p04    -   paletted, file-gamma = 0.45
        g05n3p04    -   paletted, file-gamma = 0.55
        g07n3p04    -   paletted, file-gamma = 0.70
        g10n3p04    -   paletted, file-gamma = 1.00
        g25n3p04    -   paletted, file-gamma = 2.50



3.7     Filtering test files
----------------------------

        f00n0g08    -   grayscale, no interlacing, filter-type 0 
        f01n0g08    -   grayscale, no interlacing, filter-type 1
        f02n0g08    -   grayscale, no interlacing, filter-type 2
        f03n0g08    -   grayscale, no interlacing, filter-type 3
        f04n0g08    -   grayscale, no interlacing, filter-type 4
        f00n2c08    -   color, no interlacing, filter-type 0 
        f01n2c08    -   color, no interlacing, filter-type 1
        f02n2c08    -   color, no interlacing, filter-type 2
        f03n2c08    -   color, no interlacing, filter-type 3
        f04n2c08    -   color, no interlacing, filter-type 4



3.8     Additional palette chunk test files
-------------------------------------------

        pp0n2c16    -   six-cube palette-chunk in true-color image
        pp0n6a08    -   six-cube palette-chunk in true-color+alpha image
        ps1n0g08    -   six-cube suggested palette (1 byte) in grayscale image
        ps1n2c16    -   six-cube suggested palette (1 byte) in true-color image
        ps2n0g08    -   six-cube suggested palette (2 bytes) in grayscale image
        ps2n2c16    -   six-cube suggested palette (2 bytes) in true-color image



3.9     Ancillary chunks test files
-----------------------------------

        cs5n2c08    -   color, 5 significant bits
        cs8n2c08    -   color, 8 significant bits (reference)
        cs3n2c16    -   color, 13 significant bits
        cs3n3p08    -   paletted, 3 significant bits
        cs5n3p08    -   paletted, 5 significant bits
        cs8n3p08    -   paletted, 8 significant bits (reference)

        
        cdfn2c08    -   physical pixel dimensions, 8x32 flat pixels
        cdhn2c08    -   physical pixel dimensions, 32x8 high pixels
        cdsn2c08    -   physical pixel dimensions, 8x8 square pixels
        cdun2c08    -   physical pixel dimensions, 1000 pixels per 1 meter

        
        ccwn2c08    -   chroma chunk w:0.3127,0.3290 r:0.64,0.33 g:0.30,0.60 b:0.15,0.06
        ccwn3p08    -   chroma chunk w:0.3127,0.3290 r:0.64,0.33 g:0.30,0.60 b:0.15,0.06

        ch1n3p04    -   histogram 15 colors
        ch2n3p08    -   histogram 256 colors

        cm7n0g04    -   modification time, 01-jan-1970 00:00:00
        cm9n0g04    -   modification time, 31-dec-1999 23:59:59
        cm0n0g04    -   modification time, 01-jan-2000 12:34:56

        ct0n0g04    -   no textual data
        
        ct1n0g04    -   with textual data
        ctzn0g04    -   with compressed textual data



3.10    Chunk ordering
----------------------

        oi1n0g16    -   grayscale mother image with 1 idat-chunk
        oi2n0g16    -   grayscale image with 2 idat-chunks
        oi4n0g16    -   grayscale image with 4 unequal sized idat-chunks
        oi9n0g16    -   grayscale image with all idat-chunks length one
        oi1n2c16    -   color mother image with 1 idat-chunk
        oi2n2c16    -   color image with 2 idat-chunks
        oi4n2c16    -   color image with 4 unequal sized idat-chunks
        oi9n2c16    -   color image with all idat-chunks length one



3.11    Compression level
-------------------------

        z00n2c08    -   color, no interlacing, compression level 0 (none)
        z03n2c08    -   color, no interlacing, compression level 3
        z06n2c08    -   color, no interlacing, compression level 6 (default)
        z09n2c08    -   color, no interlacing, compression level 9 (maximum)



3.12    Corrupted files
-----------------------

        x00n0g01    -   empty 0x0 grayscale file 
        xcrn0g04    -   added cr bytes
        xlfn0g04    -   converted cr bytes to lf and removed all NULs