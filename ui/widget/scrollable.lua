-- A widget that provides its child with infinite size
-- along the given scroll axis and adds a scrollbar.
local Scrollable = {}

local Vector = require("brinevector")
local dume = require("dume")

function Scrollable:new(scrollAxis, child, barWidth)
    barWidth = barWidth or 5
    local o = {
        scrollAxis = scrollAxis,
        crossAxis = dume.cross(scrollAxis),
        children = { child },
        child = child,
        barWidth = barWidth,
        state = {
            scrollPos = 0,
            grabbed = false,
            hovered = false,
        }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Scrollable:getBarRect()
    if self.scrollAxis == dume.Axis.Vertical then
        return {
            pos = Vector(self.size.x - self.barWidth, 0),
            size = Vector(self.barWidth, self.size.y),
        }
    else
        return {
            pos = Vector(0, self.size.y - self.barWidth),
            size = Vector(self.size.x, self.barWidth)
        }
    end
end

function Scrollable:barContains(pos)
    local rect = self:getBarRect()
    return dume.rectContains(rect.pos, rect.size, pos)
end

function Scrollable:handleEvent(event, cv)
    if event.type == dume.EventType.MouseClick then
        self.state.grabbed = (event.action == dume.Action.Press or event.action == dume.Action.Repeat) and self:barContains(event.pos)
    end

    if event.type == dume.EventType.CursorMove and self.state.grabbed then
        self.state.scrollPos = event.pos[self.scrollAxis]
    end

    if event.type == dume.EventType.CursorMove then
        self.state.hovered = self:barContains(event.pos)
    end

    self:invokeChildrenEvents(event, cv)
end

function Scrollable:layout(maxSize, cv)
    local childMaxSize = Vector(maxSize.x, maxSize.y)
    childMaxSize[self.scrollAxis] = math.huge
    self.child:layout(childMaxSize, cv)

    self.child.pos = Vector(0, 0)
    self.child.pos[self.scrollAxis] = -self.state.scrollPos

    self.size = maxSize
    self.size[self.crossAxis] = self.child.size[self.crossAxis]
end

function Scrollable:paint(cv)
    local style = self.style.scrollable

    cv:setScissorRect(Vector(0, 0), self.size)
    self:paintChildren(cv)
    cv:clearScissor()

    -- Scrollbar
    local scrollbar = self:getBarRect()
    cv:beginPath()
    cv:rect(scrollbar.pos, scrollbar.size)

    if self.state.grabbed then
        cv:solidColor(style.grabbedBarColor)
    elseif self.state.hovered then
        cv:solidColor(style.hoveredBarColor)
    else
        cv:solidColor(style.barColor)
    end
    cv:fill()
end

return Scrollable