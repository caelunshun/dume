package.path = "ui/?.lua"

local dume = require("dume")
local Vector = require("brinevector")
local Text = require("widget/text")
local Flex = require("widget/flex")
local Center = require("widget/center")
local Image = require("widget/image")
local Button = require("widget/button")
local ProgressBar = require("widget/progress_bar")
local Scrollable = require("widget/scrollable")
local Padding = require("widget/padding")
local Tooltip = require("widget/tooltip")
local Container = require("widget/container")

local ui = dume.UI:new(cv)

ui.style = {
    defaultTextStyle = {
        family = "Merriweather",
        size = 12,
        weight = dume.FontWeight.Normal,
        style = dume.FontStyle.Normal,
        color = dume.rgb(255, 255, 255),
    },
    backgroundColor = dume.rgb(30, 30, 30),
    borderWidth = 2,
    borderColor = dume.rgb(80, 80, 80),
    borderRadius = 5,
    hovered = {
        backgroundColor = dume.rgb(40, 40, 40),
    },
    pressed = {
        backgroundColor = dume.rgb(50, 50, 50),
        borderColor = dume.rgb(0, 169, 206),
    },
    progressBar = {
        backgroundColor = dume.rgb(0, 0, 0),
        borderColor = dume.rgb(30, 30, 30),
        borderRadius = 0,
        borderWidth = 1,
        progressColor = dume.rgb(151, 215, 0),
        positivePredictedProgressColor = dume.rgb(39, 159, 0),
        negativePredictedProgressColor = dume.rgb(207, 69, 32)
    },
    scrollable = {
        barColor = dume.rgb(30, 30, 30),
        hoveredBarColor = dume.rgb(50, 50, 50),
        grabbedBarColor = dume.rgb(80, 80, 80),
    }
}

local text1 = Text:new("@size{30}{I am @bold{Dume}. @icon{gradient} I am the @italic{%bendu}.}", { bendu = "Bendu" })

local text2 = Text:new("@size{14}{@italic{Q.E.D.}}", {}, { alignV = dume.Align.Center })

local progress = ProgressBar:new(Vector(600, 30), function()
    return (math.sin(os.clock()) + 1) / 2
end, function()
    -- derivative of progress function
    return math.min(math.max(math.cos(os.clock()) / 2 + (math.sin(os.clock()) + 1) / 2, 0), 1)
end, Text:new("@size{20}{Progress}", {}, {alignH = dume.Align.Center}))

local list = Flex:column()
for i=1,20 do
    list:addFixedChild(Text:new("Number %i", {i=i}))
end

local root = Flex:column()
root:setCrossAlign(dume.Align.Center)
root:addFlexChild(Button:new(text1, function()
    print("Clicked!")
end), 1)
root:addFlexChild(Scrollable:new(dume.Axis.Vertical, Padding:new(list, 20)), 2)
root:addFlexChild(text2, 1)
root:addFixedChild(progress)

local imageOverlayText = Text:new("@size{24}{Some smoke.}")
local imageOverlay = Tooltip:new(imageOverlayText, Container:new(Text:new("Some text over an image!")))
root:addFixedChild(Image:new("smoke", 300, Center:new(imageOverlay)))

ui:createWindow("main", Vector(0, 0), Vector(1920 / 2, 1080 / 2), root)

function draw()
    ui:render()
end

function handleEvent(event)
    -- convert tables to Vector
    if event.pos ~= nil then
        event.pos = Vector(event.pos.x, event.pos.y)
    end
    if event.offset ~= nil then
        event.offset = Vector(event.offset.x, event.offset.y)
    end

    ui:handleEvent(event)
end
