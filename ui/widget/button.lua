-- A high-level widget for a pressable icon.
local Button = {}

local Container = require("widget/container")
local Clickable = require("widget/clickable")
local Center = require("widget/center")

function Button:new(child, pressedCallback)
    local container = Container:new(Center:new(child))
    container.fillParent = true
    table.insert(container.classes, "button")
    local clickable = Clickable:new(container, pressedCallback)
    clickable.fillParent = true
    local o = {
        children = {
            clickable
        },
        classes = { "button" }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

return Button
