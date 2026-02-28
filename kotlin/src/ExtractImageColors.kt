import quantize.QuantizerCelebi
import score.Score
import java.awt.image.BufferedImage
import java.io.File
import java.util.Locale
import javax.imageio.ImageIO

fun extractImageColors() {
    val imageNames = listOf(
        "cake.png",
        "desk.png",
        "island.png",
        "river.png",
        "santorini.png",
        "tree.png",
    )

    val maxColorSettings = listOf(
        128,
        32,
        16
    )
    val desiredCountSettings = listOf(
        4,
        1
    )

    val results = mutableListOf<String>()

    for (name in imageNames) {
        val file = File("../tests/assets/img/$name")
        if (!file.exists()) {
            println("Skipping $name: File not found")
            continue
        }

        println("Extracting reference for $name...")
        val pixels = extractRawPixels(file)

        for (maxColors in maxColorSettings) {
            // Step 1: Quantize (Returns Map<Int, Int> i.e., Argb -> Count)
            val quantizerResult = QuantizerCelebi.quantize(pixels, maxColors)

            for (desiredCount in desiredCountSettings) {
                // Step 2: Score (Returns List<Int> i.e., List<Argb>)
                val seeds = Score.score(quantizerResult, desiredCount)

                val hexSeeds = seeds.joinToString(", ") {
                    String.format(Locale.US, "\"0x%08X\"", it)
                }

                val entry = """
                {
                    "image": "$name",
                    "settings": {
                        "max_colors": $maxColors,
                        "desired_count": $desiredCount
                    },
                    "seeds": [$hexSeeds]
                }
                """.trimIndent()

                results.add(entry)
            }
        }
    }

    val out = File("../tests/assets/json/reference_extraction.json")
    out.writeText("[\n" + results.joinToString(",\n") + "\n]\n")
    println("Done! Reference file generated.")
}

/**
 * Loads the image exactly as it is on disk.
 * Forces it into INT_ARGB format to ensure the alpha channel is 0xFF
 * and bits are ordered 0xAARRGGBB.
 */
fun extractRawPixels(file: File): IntArray {
    val inputImage = ImageIO.read(file)
    val width = inputImage.width
    val height = inputImage.height

    // We create a new buffer in a specific format to guarantee
    // that pixel data is not indexed or in a strange color space.
    val argbImage = BufferedImage(width, height, BufferedImage.TYPE_INT_ARGB)
    val g = argbImage.createGraphics()
    g.drawImage(inputImage, 0, 0, null)
    g.dispose()

    val pixels = IntArray(width * height)
    argbImage.getRGB(0, 0, width, height, pixels, 0, width)
    return pixels
}