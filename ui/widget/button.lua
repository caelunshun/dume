-- A high-level widget for a pressable icon.
local Button = {}

local Container = require("widget/container")
local Clickable = require("widget/clickable")
local Center = require("widget/center")

function Button:new(child, pressedCallback)
    local container = Container:new(child)

    local classes = { "button", "container" }
    container.classes = classes
    local clickable = Clickable:new(container, pressedCallback)
    local o = {
        children = {
            clickable
        },
        classes = classes,
        container = container,
        clickable = clickable,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Button:init()
    if self.fillParent or self.style.minSize then
        self.container.fillParent = true
        self.clickable.fillParent = true
        self.container.child = Center:new(self.container.child)
        self.container.children = { self.container.child }
    end
end

return Button
