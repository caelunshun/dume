-- A widget that displays a percent progress.
-- Progress is computed from a callback.
local ProgressBar = {}

local Vector = require("brinevector")

-- `progressFunction` should return a number on [0, 1].
-- `child` is optional but allows widgets to be rendered inside
-- the progress bar.
-- `predictedProgressFunction` is optional and displays an additional
-- portion on the bar to represent predicted progress.
function ProgressBar:new(size, progressFunction, predictedProgressFunction, child)
    local o = {
        children = {},
        params = {
            progressFunction = progressFunction,
            predictedProgressFunction = predictedProgressFunction,
            size = size,
        },
        classes = { "progressBar" }
    }
    if child ~= nil then o.children[1] = child end
    setmetatable(o, self)
    self.__index = self
    return o
end

function ProgressBar:paint(cv)
    local style = self.style
    local progress = self.params.progressFunction()
    local predictedProgress = 0
    if self.params.predictedProgressFunction ~= nil then
        predictedProgress = self.params.predictedProgressFunction()
    end

    progress = math.clamp(progress, 0, 1)
    predictedProgress = math.clamp(predictedProgress, 0, 1)

    cv:beginPath()
    cv:roundedRect(Vector(0, 0), self.size, style.borderRadius)

    if style.borderColor and style.borderWidth then
        cv:strokeWidth(style.borderWidth)
        cv:solidColor(style.borderColor)
        cv:stroke()
    end

    cv:solidColor(style.backgroundColor)
    cv:fill()

    -- Positive predicted progress
    if predictedProgress > progress and self.params.predictedProgressFunction ~= nil then
        cv:beginPath()
        cv:roundedRect(Vector(0, 0), Vector(self.size.x * predictedProgress, self.size.y), style.borderRadius)
        cv:solidColor(style.positivePredictedProgressColor)
        cv:fill()
    end

    -- Progress
    cv:beginPath()
    cv:roundedRect(Vector(0, 0), Vector(self.size.x * progress, self.size.y), style.borderRadius)
    cv:solidColor(style.progressColor)
    cv:fill()

    -- Negative predicted progress
    if predictedProgress < progress and self.params.predictedProgressFunction ~= nil then
        cv:beginPath()
        local pos = Vector(self.size.x * predictedProgress, 0)
        cv:roundedRect(pos, Vector(self.size.x * progress, self.size.y) - pos, style.borderRadius)
        cv:solidColor(style.negativePredictedProgressColor)
        cv:fill()
    end

    self:paintChildren(cv)
end

function ProgressBar:layout(maxSize, cv)
    self.size = self.params.size

    if self.size.x > maxSize.x then self.size.x = maxSize.x end
    if self.size.y > maxSize.y then self.size.y = maxSize.y end

    self:layoutChildren(self.size, cv)
end

return ProgressBar
