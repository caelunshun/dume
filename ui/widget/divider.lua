-- Draws a horizontal line.
local Divider = {}

local Vector = require("brinevector")

function Divider:new(width)
    local o = {
        params = {
            width = width,
        },
        classes = { "divider" }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Divider:layout(maxSize)
    self.size = Vector(
            maxSize.x,
            self.params.width
    )
end

function Divider:paint(cv)
    cv:beginPath()
    cv:moveTo(Vector(0, 0))
    cv:lineTo(Vector(self.size.x, 0))
    cv:strokeWidth(self.params.width)
    cv:solidColor(self.style.color)
    cv:stroke()
end

return Divider
