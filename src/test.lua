-- Pixels Sandbox

-- Setup the program!
function init()
    t = 0
end

-- Update the program, at 60FPS!
function update(dt)
    print("Updating from lua, dt is ", dt)
end

-- Draw sprites, tileset at 60FPS (lossy).
function draw()
    clear(math.abs(math.sin(t)), 0.0, 0.0, 0.0)
end
