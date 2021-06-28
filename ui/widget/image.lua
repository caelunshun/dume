-- A widget that renders an image.
--
-- Renders the image with the given ID registered in the canvas
-- sprite store.
--
-- The size represents the width, in logical pixels, of the image.
-- If the size is set to nil, the image will grow to fill its parent.
local Image = {}

local Vector = require("brinevector")

function Image:new(name, size, child)
    local o = { params = { name = name, size = size } }
    if child ~= nil then
        o.params.child = child
        o.children = { child }
    end
    setmetatable(o, self)
    self.__index = self
    return o
end

function Image:paint(cv)
    cv:drawSprite(self.params.name, Vector(0, 0), self.size.x)
    self:paintChildren(cv)
end

function Image:layout(maxSize, cv)
    local spriteSize = Vector(0, 0)
    cv:getSpriteSize(self.params.name, spriteSize)
    local aspect = spriteSize.y / spriteSize.x
    self.size = Vector(0, 0)

    local realMaxSize = Vector(maxSize.x, maxSize.y)
    if maxSize.x * aspect > maxSize.y then realMaxSize.x = maxSize.y / aspect end

    if self.params.size ~= nil then
        self.size.x = self.params.size
    else
        self.size.x = realMaxSize.x
    end
    self.size.y = self.size.x * aspect
    self:layoutChildren(self.size.copy, cv)
end

return Image
