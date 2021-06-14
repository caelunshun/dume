-- A widget that wraps a child and draws a background color.
local Container = {}

local dume = require("dume")
local Vector = require("brinevector")

function Container:new(child)
    local o = {
        params = { child = child }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Container:paint(cv)
    cv:beginPath()
    cv:rect(self.pos, self.size)
    cv:solidColor(self.style.backgroundColor)
    cv:fill()
    self:paintChildren(cv)
end

function Container:build()
    return {self.child}
end

return Container
