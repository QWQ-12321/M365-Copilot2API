//go:build ignore

// 生成 Tauri 桌面应用所需的图标文件（纯 Go 标准库，无需 Pillow）
// 用法: go run scripts/generate_icons.go
package main

import (
	"image"
	"image/color"
	"image/draw"
	"image/png"
	"os"
	"path/filepath"
)

var (
	primary = color.RGBA{0, 120, 212, 255}
	bg      = color.RGBA{243, 242, 241, 255}
)

func main() {
	root, _ := os.Getwd()
	iconsDir := filepath.Join(root, "src-tauri", "icons")
	os.MkdirAll(iconsDir, 0755)

	// 32x32 — 托盘图标
	img32 := makeIcon(32)
	savePng(img32, filepath.Join(iconsDir, "32x32.png"))

	// 128x128
	img128 := makeIcon(128)
	savePng(img128, filepath.Join(iconsDir, "128x128.png"))

	// 128x128@2x (256)
	img256 := makeIcon(256)
	savePng(img256, filepath.Join(iconsDir, "128x128@2x.png"))

	// icon.ico — Windows ICO（多尺寸）
	makeIco(iconsDir, img256)

	// icon.icns — macOS（PNG 占位，CI 可替换）
	savePng(img256, filepath.Join(iconsDir, "icon.icns"))

	println("\n图标生成完成！")
}

func savePng(img *image.RGBA, path string) {
	f, err := os.Create(path)
	if err != nil {
		panic(err)
	}
	defer f.Close()
	png.Encode(f, img)
	println("  已生成:", filepath.Base(path))
}

func makeIcon(size int) *image.RGBA {
	img := image.NewRGBA(image.Rect(0, 0, size, size))

	// 透明背景
	draw.Draw(img, img.Bounds(), &image.Uniform{color.Transparent}, image.Point{}, draw.Src)

	// 圆角矩形背景
	r := size / 5
	b := image.Rect(2, 2, size-2, size-2)
	fillRoundedRect(img, b, r, bg)
	fillRoundedRect(img, b, r, primary) // 细边框效果 — 实际用稍小的内矩形

	// 三条横线
	lineW := size / 6
	margin := size / 5
	spacing := size / 8
	totalH := lineW*3 + spacing*2
	startY := (size - totalH) / 2

	for i := 0; i < 3; i++ {
		y := startY + i*(lineW+spacing)
		lw := size - margin*2 - i*size/10
		x := margin + i*size/20
		rr := image.Rect(x, y, x+lw, y+lineW)
		fillRoundedRect(img, rr, lineW/2, primary)
	}

	return img
}

func fillRoundedRect(img *image.RGBA, r image.Rectangle, radius int, c color.Color) {
	for y := r.Min.Y; y < r.Max.Y; y++ {
		for x := r.Min.X; x < r.Max.X; x++ {
			if isInRoundedRect(x, y, r, radius) {
				img.Set(x, y, c)
			}
		}
	}
}

func isInRoundedRect(x, y int, r image.Rectangle, radius int) bool {
	cx := r.Min.X + radius
	cy := r.Min.Y + radius
	// 检查是否在四个角的圆内或矩形主体内
	inBody := (x >= r.Min.X+radius && x < r.Max.X-radius) ||
		(y >= r.Min.Y+radius && y < r.Max.Y-radius)
	if inBody {
		return true
	}
	// 检查四角
	corners := [][2]int{
		{cx, cy},
		{r.Max.X - radius - 1, cy},
		{cx, r.Max.Y - radius - 1},
		{r.Max.X - radius - 1, r.Max.Y - radius - 1},
	}
	for _, cc := range corners {
		dx := x - cc[0]
		dy := y - cc[1]
		if dx*dx+dy*dy <= radius*radius {
			return true
		}
	}
	return false
}

func makeIco(iconsDir string, big *image.RGBA) {
	// 写入包含 256x256 PNG 的 ICO 文件
	// ICO 格式：ICONDIR + ICONDIRENTRY + PNG data
	pngData := pngBytes(big)

	f, err := os.Create(filepath.Join(iconsDir, "icon.ico"))
	if err != nil {
		panic(err)
	}
	defer f.Close()

	// ICONDIR header (6 bytes)
	f.Write([]byte{0, 0, 1, 0, 1, 0}) // reserved, type=1 (ICO), count=1

	// ICONDIRENTRY (16 bytes)
	// width=0 means 256, height=0 means 256
	entry := []byte{
		0,          // width (0 = 256)
		0,          // height (0 = 256)
		0,          // color palette (0 = more than 256)
		0,          // reserved
		1, 0,       // color planes
		32, 0,      // bits per pixel
	}
	// size of data
	sizeBytes := uint32ToLE(uint32(len(pngData)))
	entry = append(entry, sizeBytes...)
	// offset of data
	entry = append(entry, uint32ToLE(22)...) // 6 + 16 = 22

	f.Write(entry)
	f.Write(pngData)
	println("  已生成: icon.ico")
}

func pngBytes(img *image.RGBA) []byte {
	var buf []byte
	// 用临时文件获取 PNG 字节
	tmpfile, _ := os.CreateTemp("", "icon*.png")
	defer os.Remove(tmpfile.Name())
	png.Encode(tmpfile, img)
	tmpfile.Close()
	buf, _ = os.ReadFile(tmpfile.Name())
	return buf
}

func uint32ToLE(v uint32) []byte {
	return []byte{
		byte(v),
		byte(v >> 8),
		byte(v >> 16),
		byte(v >> 24),
	}
}
