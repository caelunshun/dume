-- A widget that renders an image.
--
-- Renders the image with the given ID registered in the canvas
-- sprite store.
--
-- The size represents the width, in logical pixels, of the image.
local Image = {}

local Vector = require("brinevector")

function Image:new(name, size, child)
    size = size or 100
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
    cv:drawSprite(self.params.name, Vector(0, 0), self.params.size)
    self:paintChildren(cv)
end

function Image:layout(maxSize, cv)
    local spriteSize = Vector(0, 0)
    cv:getSpriteSize(self.params.name, spriteSize)
    local aspect = spriteSize.y / spriteSize.x
    self.size = Vector(0, 0)
    self.size.x = self.params.size
    self.size.y = self.params.size * aspect
    self:layoutChildren(self.size, cv)
end

return Image
