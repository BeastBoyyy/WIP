# Rice Studio (native): design

The web prototype's brief and concept still hold (`../rice-studio/DESIGN.md`): the app is the demo,
and changes *develop* from the point you touched. The native build adds a GPU stage, after the
spatial work on activetheory.net: things live in one 3D space and the camera travels between them.

## The stage (wgpu, behind the Slint UI)
- **Gallery:** the ten theme cards are glass slabs on a helix around a tesseract rotating in 4D.
  Wheel, arrow keys or a click spiral the camera to the next card. The focused card sits right of
  centre; the previous one hangs above left, so the helix reads at a glance.
- **Glass:** rounded edges, a rim light in the theme's accent (brighter on hover and focus), a slow
  sheen, chromatic split near the rim, a darkened mirror on the back face.
- **Dust:** each nearby card sheds particles coloured by sampling its own face, so Inferno sheds
  embers and Marquee sheds red and paper.
- **Tesseract (placeholder for the 3D logo):** 32 edges projected from 4D; the fourth coordinate shows as a blend between the
  theme's two accents.
- **Fog:** a domain-warped nebula tinted to the focused theme, with far stars and grain.
- **Customize / Wallpapers:** the focused card flies to the left of the side panel and comes apart
  into wallpaper, bar and clock layers, at a resting three-quarter turn that follows the pointer.
- **Claude:** the ring opens out and dims, and the camera moves in on the tesseract.
- **Ink veil:** variant switches and a finished Apply flood the screen with the theme's ink from a
  point, then it dries away in blotches.
- **Apply:** a bright line develops the focused card top to bottom while the real work runs.

## Motion
| Role | Treatment |
|---|---|
| Card focus | damped spring, slight overshoot |
| Page change | camera and desk springs, panels fade and slide 48 px |
| Theme name | letters rise one by one in the theme's own display face |
| Nav | highlight slides to the current page |
| Intro | camera pulls back from the tesseract as the cards are dealt out, nearest first |

## Performance
Only the stage animates continuously. It renders at 60 fps while anything moves, 30 fps for 10 s
after, 12 fps until a minute of no input, then 4 fps. It also drops to 4 fps whenever another window is
in front, and stops while minimised.

## Reduced motion
When Windows has "Show animations" off: no intro, no veil, no dust, springs land instantly, and
the stage's clock runs at a quarter speed.

## Rules kept from Hallmark
Tokens in one `Ui` global; sentence case; no tracked caps or eyebrows; no middle-dot strings;
focus rings on every control; accent text colour picked for contrast; no invented numbers.
