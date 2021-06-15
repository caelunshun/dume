local text = cv:parseTextMarkup("@size{30}{I am @bold{Dume}. @italic{I am the %bendu.}}", {bendu = "Bendu"})
local paragraph = cv:createParagraph(text, {
    alignH = 1,
    alignV = 1,
    baseline = 0,
    lineBreaks = true,
    maxDimensions = { x = 1920 / 2, y = 1080 / 2 }
})

function draw()
    cv:drawSprite("sprite", {x = 0, y = 0}, 400)

    cv:beginPath()
    cv:moveTo({x = 0, y = 0})
    cv:lineTo({x = 450, y = 450})
    cv:quadTo({x = 500, y = 200}, { x = 600, y = 300})
    cv:linearGradient({x = 0, y = 0}, {x = 1920 / 2, y = 1080 / 2}, {96, 45, 226, 255}, {255, 255, 255, 255})
    cv:fill()

    cv:drawParagraph(paragraph, { x = 0, y = 0 })
end