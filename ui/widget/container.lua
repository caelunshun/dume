-- A widget that wraps a child and draws a background color and border.
local Container = {}

local dume = require("dume")
local Vector = require("brinevector")

function Container:new(child)
    local o = {
        children = {child}, child = child, classes = { "container" }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Container:paint(cv)
    cv:beginPath()
    local pos
    if self.fillParent then
        pos = Vector(0, 0)
    else
        pos = self.offsetFromParent
    end
    cv:roundedRect(pos, self.size, self.style.borderRadius or 0)
    if self.style.backgroundColor then
        cv:solidColor(self.style.backgroundColor)
        cv:fill()
    end
    if self.style.borderColor and self.style.borderWidth then
        cv:solidColor(self.style.borderColor)
        cv:strokeWidth(self.style.borderWidth)
        cv:stroke()
    end
    self:paintChildren(cv)
end

return Container
