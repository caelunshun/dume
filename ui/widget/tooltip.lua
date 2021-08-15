-- Causes a tooltip to be rendered when the child is hovered.
local Tooltip = {}

local Vector = require("brinevector")
local dume = require("dume")

function Tooltip:new(child, tooltip)
    local o = {
        children = {child, tooltip},
        child = child,
        tooltip = tooltip,
        state = { showing = false, cursorPos = Vector(0, 0) },
        classes = { "tooltip" }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Tooltip:handleEvent(event, cv)
    self:invokeChildrenEvents(event, cv)
    if event.type == dume.EventType.CursorMove then
        self.state.showing = self:contains(event.pos)
        self.state.cursorPos = event.pos
    end
end

function Tooltip:layout(maxSize, cv)
    self.child:layout(maxSize, cv)
    self.child.pos = Vector(0, 0)
    self.size = self.child.size

    self.tooltip:layout(Vector(cv:getWidth(), cv:getHeight()), cv)
    self.tooltip.pos = Vector(-self.tooltip.size.x - 10, -self.tooltip.size.y / 2 + self.child.size.y / 2)
end

function Tooltip:paint(cv)
    self.child:paint(cv)

    if self.state.showing then
        cv:translate(self.tooltip.pos)
        self.tooltip:paint(cv)
        cv:translate(-self.tooltip.pos)
    end
end

return Tooltip

