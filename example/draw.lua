package.path = "ui/?.lua"

local dume = require("dume")
local Vector = require("brinevector")
local Text = require("widget/text")
local Flex = require("widget/flex")
local Container = require("widget/container")
local Image = require("widget/image")
local Button = require("widget/button")
local StyleModifier = require("widget/style_modifier")

local ui = dume.UI:new(cv)

ui.style = {
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
    }
}

local text1 = Text:new("@size{30}{I am @bold{Dume}. @icon{gradient} I am the @italic{%bendu}.}", { bendu = "Bendu" })

local text2 = Text:new("@size{14}{@italic{Q.E.D.}}", {})

local text3 = Text:new("I have spoken.", {}, {baseline = dume.Baseline.Bottom})
local text4 = Text:new("@size{20}{@color{rgb(255,40,40)}{ I am you.}}", {}, {baseline = dume.Baseline.Bottom})

local nested = Flex:row()
nested:setMainAlign(dume.Align.Center)
nested:setCrossAlign(dume.Align.Center)
nested:addFixedChild(text3)
nested:addFixedChild(text4)

local root = Flex:column()
root:setCrossAlign(dume.Align.Center)
root:addFlexChild(Button:new(text1, function()
    print("Clicked!")
end), 1)
root:addFlexChild(text2, 1)
root:addFlexChild(Container:new(nested), 1)
root:addFixedChild(Image:new("smoke", 600))

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
