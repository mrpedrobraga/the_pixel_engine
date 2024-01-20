-- Pixels Sandbox

-- Setup the program!
function init()
    t = 0
end

-- Update the program, at 60FPS!
function update(dt)

end

function input(kind, pressed)
    print(kind, pressed)
end

-- Draw sprites, tileset at 60FPS (lossy).
function draw()
    clear(0, 0.5 + 0.5 * math.sin(t * math.pi * 2.0), 0, 0);
end
