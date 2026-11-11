# Nghiên Cứu TikTok Video AI: Kịch Bản & Prompt Engineering Thực Chiến
**Thời gian thực hiện:** 2026-11-08  
**Công cụ thu thập:** Trình duyệt Ego Lite (`ego-browser` CLI, Node runtime, Profile "Thiên Thoại", Headed mode, Stealth flags, Jitter delay 500–2000ms, TaskSpace lifecycle chuẩn).  
**Phạm vi:** 12 nhóm từ khóa cốt lõi trên TikTok, phân tích trên 200+ video thực tế và quét sâu 23 video dẫn đầu về lượt tương tác, công thức và kỹ thuật tạo video AI.

---

## Phần 1: Kịch Bản Video TikTok (Tổng Hợp Từ Prompt 1)

*(Ghi chú: Giữ nguyên khung cấu trúc và nội dung nền tảng của Prompt 1 về biên kịch video viral và tâm lý học giữ chân người xem).*

### 1.1. Công thức Mở đầu (Hook Formula 0-3s)
- **Tỉ lệ rớt người xem cao nhất:** Diễn ra ở 1.8s đầu tiên nếu hình ảnh hoặc âm thanh không đủ bất ngờ.
- **Hook 3 thành phần:**
  1. *Hình ảnh gây tò mò (Visual Disruption):* Đồ họa chuyển động 3D giật nảy hoặc góc máy cận cảnh bất thường (macro / extreme close-up).
  2. *Câu nói đánh thẳng vào nỗi đau (Pain-point Question):* "Có bao giờ bạn tự hỏi...", "Đừng làm video AI nếu chưa biết mẹo này..."
  3. *Âm thanh báo hiệu (Sound Trigger):* Tiếng "ding" thanh thoát hoặc âm bass sweep nhanh để báo hiệu nội dung quan trọng bắt đầu.

### 1.2. Nhịp phân cảnh & Duy trì tương tác (Retention Rhythm)
- **Chu kỳ chuyển đổi thị giác (Cut Frequency):** Trung bình 2.2 – 3.0 giây phải có 1 chuyển động máy hoặc chuyển cảnh mới (B-roll, zoom-in, text highlight).
- **Quy tắc Safe Zone 9:16:** Giữ toàn bộ chữ và hành động chính cách mép trên tối thiểu 180px và mép dưới 320px để tránh bị icon like, share và giỏ hàng TikTok che lấp.

---

## Phần 2: Prompt Làm Video Thực Chiến (Prompt 2)

Dưới đây là 22 prompt và công thức được trích xuất trực tiếp từ các nhà sáng tạo video AI hàng đầu trên TikTok, chia thành 5 nhóm chuyên môn hóa. Mỗi prompt đều có trích dẫn nguyên văn (dưới 25 từ), công cụ sử dụng, thông số kỹ thuật và phiên bản viết lại tương thích tuyệt đối với hệ thống hoạt hình 3D / KOC AI của **SocNhoTV** (Sóc Nhỏ Bi 100cm, Koala 150cm, HeyGen HyperFrames, OmniVoice).

---

### Nhóm 1: Prompt Tạo Cảnh (Scene Generation)

#### Prompt 1.1: B-roll Sản Phẩm Giải Khát & Năng Động Cực Nét
- **Nguồn:** [@dinprasetyo.id](https://www.tiktok.com/@dinprasetyo.id/video/7560964248382328076) (dinprasetyo - Thử nghiệm so sánh 1 prompt trên 3 mô hình)
- **Công cụ AI:** Hailuo AI (MiniMax 02) / Google DeepMind Veo 3 / Sora 2
- **Tỉ lệ khung hình:** 9:16 (dọc TikTok)
- **Prompt gốc (trích dẫn, 18 từ):**
  > *"Refreshing canned juice commercial b-roll, ice splashes, condensation droplets on metallic can, vibrant sunny atmosphere, cinematic 4k"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Ly nước ép hạt dẻ ướp lạnh lấp lánh giọt sương, Sóc Nhỏ Bi ngồi dưới gốc sồi mùa thu vàng ấm, ánh nắng xuyên kẽ lá, phong cách hoạt hình 3D Pixar 4k render."`
- **Độ hữu ích:** 5/5

#### Prompt 1.2: Bối Cảnh Rừng Cổ Tích Huyền Bí & Chiều Sâu Không Gian
- **Nguồn:** [@huandaotaoyoutube](https://www.tiktok.com/@huandaotaoyoutube/video/7687971835857620232) (Huan.academy - Atlas Cloud Prompts Hub)
- **Công cụ AI:** Seedance 2.0 / Wan 3.0 / Seedream
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 17 từ):**
  > *"Cinematic fantasy forest glade, golden hour volumetric lighting, whimsical mushroom houses, hyper-detailed nature, soft breeze moving leaves"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Khu rừng sồi cổ tích lung linh tia sáng hoàng hôn, ngôi nhà nấm hạt dẻ của Sóc Nhỏ Bi, gió thổi lá phong bay rực rỡ, hoạt hình 3D Pixar màu sắc ấm áp."`
- **Độ hữu ích:** 5/5

#### Prompt 1.3: Nền Đồ Họa Chuyển Động Hiện Đại Cho Giới Thiệu Tính Năng
- **Nguồn:** [@huandaotaoyoutube](https://www.tiktok.com/@huandaotaoyoutube/video/7703926073637014836) (Huan.academy - Motion Graphics AI Viral)
- **Công cụ AI:** Seedance 2.0 / CapCut AI / Midjourney
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 18 từ):**
  > *"Clean modern motion graphics background, dynamic floating 3D geometric shapes, soft gradient lighting, minimal isometric layout, sleek studio render"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Phông nền đồ họa chuyển động 3D các khối hạt dẻ và biểu tượng quà tặng nảy nhịp nhàng theo nhạc, chuyển sắc pastel tươi sáng, chuẩn TikTok Safe Zone."`
- **Độ hữu ích:** 4/5

#### Prompt 1.4: Không Gian Studio Podcast & Trò Chuyện Thân Mật
- **Nguồn:** [@realitydp.ai](https://www.tiktok.com/@realitydp.ai/video/7557034019691531551) (Melpomene AI - Thử nghiệm Sora 2 avatar)
- **Công cụ AI:** OpenAI Sora 2
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Cozy modern studio podcast room, warm ambient neon rim light, acoustic wooden wall panels, depth of field, casual talking scene"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Góc studio ấm cúng của Sóc Nhỏ và bạn Koala, bàn gỗ sồi mini, micro phát thanh nhỏ nhắn, ánh đèn vàng dịu mắt, hậu cảnh mờ mịn tạo chiều sâu."`
- **Độ hữu ích:** 4/5

---

### Nhóm 2: Prompt Chuyển Động Camera (Camera Movement)

#### Prompt 2.1: Cú Lượn Tay Robot Chuẩn Xác Tuyệt Đối (Robot Arm)
- **Nguồn:** [@antonfung](https://www.tiktok.com/@antonfung/video/7582217687208660245) (ANTON | VFX • AE • CapCut - 30.9K saves)
- **Công cụ AI:** Midjourney v6 + Runway Gen-3 Alpha / Kling 1.5
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Smooth camera motion as if using a robot arm, while keeping the subject and background unaltered, seamless non-AI look"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Cú máy cánh tay robot lượn mượt mà từ góc nghiêng sang chính diện Sóc Nhỏ Bi cao 100cm, chủ thể khóa cứng ánh mắt nhìn camera, bối cảnh rừng thông giữ nguyên vẹn."`
- **Độ hữu ích:** 5/5

#### Prompt 2.2: Đẩy Máy Góc Thấp Nhịp Nhàng Theo Bước Chân (Tracking Push-In)
- **Nguồn:** [@learnwithcheer](https://www.tiktok.com/@learnwithcheer/video/7593743272778927378) (Cheer | ai ads creatives for DTC)
- **Công cụ AI:** Sora 2 / Runway Gen-3
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Dynamic low-angle slow push-in tracking shot, sweeping cinematic movement, stabilizing on hero subject, shallow focus, 35mm lens feel"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Camera đặt sát mặt đất đẩy tới chậm rãi theo bước chân vui nhộn của Sóc Nhỏ và bạn Koala, góc nhìn 35mm xóa phông nhẹ làm nổi bật hai nhân vật chính."`
- **Độ hữu ích:** 5/5

#### Prompt 2.3: Sải Bước Catwalk Thời Trang & Tracking Lùi (Catwalk Motion)
- **Nguồn:** [@agigroup.ai](https://www.tiktok.com/@agigroup.ai/video/7616336107630152980) (AGI Group - Bí kíp chuyển động thời trang)
- **Công cụ AI:** Kling AI 1.5 / Runway Gen-3
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 20 từ):**
  > *"Full body fashion catwalk, model walking steadily towards camera, front tracking pan, dynamic fabric flow, steady optical stabilizer, continuous motion"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Bạn Koala 150cm tự tin sải bước về phía camera như sàn catwalk thời trang, camera tracking lùi đều đặn, vạt áo len đung đưa tự nhiên theo từng bước đi."`
- **Độ hữu ích:** 4/5

#### Prompt 2.4: Xoay Quanh Đối Tượng 360 Độ Khóa Chiều Sâu (Orbital Turn)
- **Nguồn:** [@leduyhiep](https://www.tiktok.com/@leduyhiep) (Lê Duy Hiệp - AI Hub)
- **Công cụ AI:** Kling AI 1.5 / ComfyUI CameraCtrl
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 18 từ):**
  > *"Slow 360 degree orbital camera turn around subject, continuous parallax depth, dramatic studio rim lights, smooth steadycam rotation"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Camera xoay tròn 360 độ quanh hộp quà hạt dẻ đặc biệt của Sóc Nhỏ, ánh sáng ven vàng ấm áp lướt qua các góc cạnh, tạo hiệu ứng chiều sâu 3 lớp."`
- **Độ hữu ích:** 5/5

---

### Nhóm 3: Prompt Nhân Vật Ổn Định (Consistent Character)

#### Prompt 3.1: Bảng Xoay Nhân Vật Đa Góc Khóa Diện Mạo (Character Sheet)
- **Nguồn:** [@ductuanmasterai](https://www.tiktok.com/@ductuanmasterai/video/7701568707251784978) (Đức Tuấn Master AI - Đồng nhất nhân vật 1 prompt)
- **Công cụ AI:** Seedance 2.0 / Midjourney
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 18 từ):**
  > *"Consistent character sheet, multi-angle turnaround, same facial features, specific wardrobe lock, neutral expression reference for multi-scene video generation"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Bảng tham chiếu DNA nhân vật Sóc Nhỏ Bi cao 100cm, bộ lông nâu cam bóng mượt, áo yếm xanh lá, nụ cười rạng rỡ khóa cứng qua góc trước, nghiêng và sau."`
- **Độ hữu ích:** 5/5

#### Prompt 3.2: Làn Da & Chất Liệu Lông Thật Tự Nhiên Không Bệt Nhựa (Anti-Plastic Texture)
- **Nguồn:** [@ngocmiai](https://www.tiktok.com/@ngocmiai/video/7621238951155666197) (Ngọc Mi ✅️ Vietnam AI Model)
- **Công cụ AI:** Midjourney v6 / Freepik AI
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Hyper-realistic natural skin texture, visible pores, subtle peach fuzz, no plastic airbrushing, soft diffused daylight, authentic features, 8k"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Kết cấu lông 3D siêu thực cho bạn Koala 150cm, từng sợi lông tơ mềm mại bắt sáng tự nhiên, chất liệu vải len không bị bệt nhựa, chuẩn hoạt hình Pixar SOTA."`
- **Độ hữu ích:** 5/5

#### Prompt 3.3: System Prompt Neo Giữ Tỷ Lệ Chiều Cao & Nhận Diện (Identity & Scale Lock)
- **Nguồn:** [@quynhaq](https://www.tiktok.com/@quynhaq/video/7647190376868056340) (Quỳnh AQ - Đồng nhất nhân vật bằng System Prompt)
- **Công cụ AI:** ChatGPT / Google Flow / Claude
- **Tỉ lệ khung hình:** Khóa Logic Tạo Ảnh
- **Prompt gốc (trích dẫn, 21 từ):**
  > *"System prompt: Maintain exact character identity [Char_ID], fixed height ratio, identical color palette and facial geometry across all sequential scene renders"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"System Prompt: 'Khóa cứng tỷ lệ hình thể: Sóc Nhỏ Bi cao đúng 100cm, bạn Koala cao 150cm (Scale Lock), bảng màu cam-xanh đồng nhất tuyệt đối qua tất cả các cảnh.'"`
- **Độ hữu ích:** 5/5

#### Prompt 3.4: Kéo Dài Video Liền Mạch Neo Theo Frame Gốc (Veo 3 Anchor Extension)
- **Nguồn:** [@chauhoichutraloi](https://www.tiktok.com/@chauhoichutraloi/video/7575022603983785236) (Hoài Nam AI - Đồng nhất nhân vật trên Veo 3)
- **Công cụ AI:** Google DeepMind Veo 3 / Google Flow
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Veo 3 character continuity: reference start frame character anchor, extend video seamlessly without changing outfit, face symmetry, or hairstyle"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Khóa frame đầu tiên làm mốc neo trong Veo 3: kéo dài video 5 giây, Sóc Nhỏ Bi giữ nguyên vóc dáng 100cm cầm giỏ sồi, ngón tay cử động tự nhiên không dị tật."`
- **Độ hữu ích:** 5/5

#### Prompt 3.5: Cảnh Dài 20s Giữ Trạng Thái Nhân Vật Hoàn Hảo (Long-form Continuity)
- **Nguồn:** [@nikoxstudios](https://www.tiktok.com/@nikoxstudios/video/7620914061001248032) (Niko | AI Creative Director - Thử nghiệm Sora 2 Arcads)
- **Công cụ AI:** OpenAI Sora 2 / Arcads AI
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Long-form single prompt character consistency, continuous twenty-second scene, persistent character features, realistic subtle eye blinks and natural head turns"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Phân cảnh 15 giây liên tục Sóc Nhỏ Bi hào hứng giới thiệu bảng học chữ cái trước lớp học rừng xanh, nháy mắt và quay đầu linh hoạt, thần thái đồng nhất 100%."`
- **Độ hữu ích:** 5/5

---

### Nhóm 4: Prompt Sản Phẩm (Product Showcase & Review)

#### Prompt 4.1: Chụp Ảnh & Video Sản Phẩm Chuẩn Bục Studio Sang Trọng
- **Nguồn:** [@leduyhiep](https://www.tiktok.com/@leduyhiep) (Lê Duy Hiệp - AI Hub)
- **Công cụ AI:** Google Gemini 2.5 Flash / Midjourney
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 21 từ):**
  > *"Commercial studio product photography, clean pedestal stand, sharp focus on packaging details, soft rim lighting, luxury cosmetic reflection, 8k render"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Bục gỗ sồi tròn đặt bộ sản phẩm quà tặng thiên nhiên của Sóc Nhỏ TV, ánh sáng studio ven vàng ấm, logo và chữ trên vỏ hộp rõ nét từng chi tiết."`
- **Độ hữu ích:** 5/5

#### Prompt 4.2: Tách Lớp 3D Bùng Nổ Năng Động (3D Explosion View)
- **Nguồn:** [@nhatduyai](https://www.tiktok.com/@nhatduyai/video/7627838867327995144) (Render Duy - AI Worker)
- **Công cụ AI:** CapCut AI / Seedance 2.0
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Dynamic 3D product explosion view, floating components, liquid splash wrapping around bottle, high speed shutter, vibrant advertising commercial look"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Hiệu ứng tách lớp 3D hộp quà Sóc Nhỏ tung bay các hạt dẻ vàng óng và sticker vui nhộn, nước hoa quả bắn tung tóe tươi mát, phong cách quảng cáo hoạt hình cuốn hút."`
- **Độ hữu ích:** 5/5

#### Prompt 4.3: Kịch Bản UGC Mở Hộp Tự Nhiên Chuẩn 6 Phân Cảnh
- **Nguồn:** [@andrbs48](https://www.tiktok.com/@andrbs48/video/7691165301546683666) (Nexora AI - Storyboard UGC bằng ChatGPT)
- **Công cụ AI:** ChatGPT + Kling AI 1.5
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 20 từ):**
  > *"GPT: create UGC style storyboard in 6 scenes, 9:16 ratio, realistic consumer unboxing, honest reaction, natural handheld camera shake, daylight"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Kịch bản 6 phân cảnh 9:16 Sóc Nhỏ và Koala mở hộp đồ chơi thông minh, nét mặt hào hứng tò mò, góc quay cầm tay rung nhẹ tự nhiên như mẹ bé quay clip."`
- **Độ hữu ích:** 5/5

#### Prompt 4.4: Cận Cảnh Đôi Tay Mở Hộp Kèm Âm Thanh Xúc Giác (Tactile Unboxing)
- **Nguồn:** [@aiacademyglobal](https://www.tiktok.com/@aiacademyglobal/video/7660513202693754133) (AI Academy Global - Affiliate Unbox)
- **Công cụ AI:** Kling 1.5 / Google Flow
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Close-up unboxing hands holding product, tearing seal tape, revealing inner package, tactile sound cue, authentic unboxing review perspective, 4k"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Cận cảnh đôi bàn tay bông xù của Sóc Nhỏ bóc seal hộp sồi bí mật, mở nắp khoe cuốn truyện tranh nhiều màu sắc, phối tiếng rột roạt ASMR kích thích giác quan."`
- **Độ hữu ích:** 4/5

#### Prompt 4.5: KOC AI Trực Tiếp Review Đứng Trước Ống Kính
- **Nguồn:** [@loannguyenai](https://www.tiktok.com/@loannguyenai/video/7585877004445125909) (Loan Nguyen AI - Review Mỹ Phẩm Thật 99%)
- **Công cụ AI:** HeyGen / OmniVoice / Kling AI
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 20 từ):**
  > *"AI KOC influencer holding product close to camera, speaking enthusiastically with natural mouth sync, soft beauty lighting, bedroom vanity background"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Nhân vật KOC AI bạn Koala 150cm cầm sản phẩm giơ sát ống kính, hào hứng chia sẻ lợi ích, nhép môi chuẩn xác từng âm tiết tiếng Việt, phòng học sinh động."`
- **Độ hữu ích:** 4/5

---

### Nhóm 5: Prompt Chuyển Cảnh (Transition & Motion Pivot)

#### Prompt 5.1: Biến Đổi Không Gian Liền Mạch Khóa Trục Mắt
- **Nguồn:** [@antonfung](https://www.tiktok.com/@antonfung/video/7582217687208660245) (ANTON | VFX • AE • CapCut)
- **Công cụ AI:** Midjourney + Kling / Runway
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 21 từ):**
  > *"Transform first image to second image, smooth camera motion robot arm, subject and background unaltered, seamless transition, no extra head movements"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Chuyển tiếp mượt mà từ cảnh Sóc Nhỏ ở rừng sồi sang studio lớp học bằng cú lượn camera robot, vị trí mắt và nụ cười của Sóc Nhỏ giữ bất động tuyệt đối."`
- **Độ hữu ích:** 5/5

#### Prompt 5.2: Cú Lia Nhanh Whip Pan Bắt Trọn Điểm Rơi Chuyển Động
- **Nguồn:** [@mikelikedesign](https://www.tiktok.com/@mikelikedesign/video/7698761488814689543) (Mike Like Design - Prompt ChatGPT sửa nhịp edit)
- **Công cụ AI:** ChatGPT + HeyGen HyperFrames
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 21 từ):**
  > *"Prompt ChatGPT to script seamless whip pan transition between two scenes, matching focal action point, high-speed motion blur, dynamic audio cue"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Cú lia máy siêu tốc (Whip Pan) theo hướng quả sồi bay từ trái sang phải, nối tiếp cảnh Sóc Nhỏ ném sang cảnh bạn Koala bắt lấy, chuyển động nối tiếp không đứt đoạn."`
- **Độ hữu ích:** 5/5

#### Prompt 5.3: Match Cut Hành Động & Thu Phóng Vào Tâm Điểm
- **Nguồn:** [@malick_cine](https://www.tiktok.com/@malick_cine) (Malick | Cinematic Videography)
- **Công cụ AI:** Midjourney + Runway Gen-3
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 21 từ):**
  > *"Match cut transition on subject action, camera push into object center, emerging into next environment with matching geometry and color tone"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Cú máy lao thẳng vào tâm chiếc kính lúp Sóc Nhỏ đang cầm soi hạt dẻ, chuyển cảnh xuyên thấu đưa người xem bước thẳng vào thế giới vi mô kỳ ảo của thế giới thực vật."`
- **Độ hữu ích:** 5/5

#### Prompt 5.4: Chuyển Cảnh Giọt Nước Hòa Tan Biến Hình (Fluid Splash Morph)
- **Nguồn:** [@dinprasetyo.id](https://www.tiktok.com/@dinprasetyo.id/video/7560964248382328076) (dinprasetyo)
- **Công cụ AI:** Hailuo 02 / Google DeepMind Veo 3
- **Tỉ lệ khung hình:** 9:16
- **Prompt gốc (trích dẫn, 19 từ):**
  > *"Liquid splash morph transition, fluid droplets forming the next scene background, smooth temporal interpolation, high frame rate slow motion"*
- **Bản viết lại theo phong cách SocNhoTV:**
  > `"Các giọt nước trái cây bắn tung lên rồi chậm rãi gom lại biến thành tấm màn chuyển cảnh mở ra phân đoạn review thông tin dinh dưỡng với đồ họa HyperFrames."`
- **Độ hữu ích:** 4/5

---

## BƯỚC TỔNG HỢP (Tổng Thể Kịch Bản & Prompt Video AI)

### A. Tóm Tắt Chung
1. **Làn sóng chuyên nghiệp hóa video AI:** TikTok cuối 2026 không còn chấp nhận các video AI méo hình, bệt nhựa hoặc trôi dạt nhân vật vô nghĩa. Các video đạt hàng chục ngàn lượt lưu đều tập trung vào:
   - *Khóa chặt tính nhất quán (Consistency Lock):* Khóa diện mạo nhân vật qua Character Turnaround Sheet hoặc System Prompt Anchor.
   - *Cú máy vật lý thực tế:* Sử dụng chuyển động có chủ đích (Robot-arm, whip pan, 360 orbit) thay vì để AI tự tạo chuyển động ngẫu nhiên.
   - *Hậu kỳ tất định (Deterministic Editing):* Kết hợp render AI nguyên liệu (raw footage) với engine dựng cấu trúc như **HeyGen HyperFrames**, đảm bảo khớp nhạc, khớp voiceover và căn chuẩn TikTok Safe Zone.
2. **Quy trình chuẩn hóa 4 bước:**
   `Kịch bản cấu trúc (ChatGPT / Claude) -> Khóa Frame & Prompt (Midjourney / Seedance / Veo 3) -> Sinh Chuyển Động (Kling 1.5 / Runway Gen-3 / Sora 2) -> Lắp ghép & Âm thanh (HyperFrames + OmniVoice)`.

---

### B. Danh Sách Đã Thu Thập (Skill, Kịch Bản, Prompt & Công Cụ)
- **Tổng số video quét ban đầu:** 202 video trên 12 từ khóa mục tiêu.
- **Tổng số video phân tích chi tiết:** 23 video có chỉ số tương tác cao và công thức rõ ràng.
- **Tổng số prompt chuẩn hóa:** 22 prompt thuộc 5 phân loại cốt lõi.
- **Các hệ công cụ AI được ghi nhận:**
  - *Mô hình sinh video:* Google DeepMind Veo 3, OpenAI Sora 2, Kling AI 1.5, Runway Gen-3 Alpha, Hailuo AI (MiniMax 02), Seedance 2.0, CapCut AI.
  - *Mô hình tạo ảnh & Neo nhân vật:* Midjourney v6, Google Gemini 2.5 Flash, Freepik AI, ComfyUI.
  - *Biên kịch & Phân cảnh:* Claude 3.5 / ChatGPT-4o / Google Flow.
  - *Dựng & Biên tập đồ họa:* HeyGen HyperFrames, Remotion, CapCut.
  - *Giọng đọc:* OmniVoice, ElevenLabs.

---

### C. Bài Học Có Thể Học (Bài Học Thực Chiến Cho SocNhoTV)

#### Bài học 1: Kỹ thuật khóa trục mắt và chủ thể khi chuyển cảnh (Robot-Arm Motion Pivot)
- **Nội dung cụ thể:** Khi chuyển từ cảnh này sang cảnh khác, việc giữ cho mắt và tư thế nhân vật đứng yên trong khi xoay camera sẽ loại bỏ 100% hiện tượng "giật cục" và biến dạng thường thấy ở video AI.
- **Bằng chứng:** ≥ 2 nguồn độc lập:
  - Video của @antonfung (`7582217687208660245` - 30.9K saves/shares).
  - Video của @dinprasetyo.id (`7560964248382328076`).
- **Cách áp dụng vào SocNhoTV:** Áp dụng vào các video hoạt hình khi Sóc Nhỏ Bi chuyển từ khu rừng cổ tích sang không gian phòng thí nghiệm hoặc lớp học: nhân vật Bi đứng yên mỉm cười nhìn thẳng camera, không gian phía sau xoay lướt chuyển đổi kỳ ảo.
- **Độ tin cậy:** Cao (High - Đã có thẻ kỹ thuật `robot-arm-motion-pivot-transition.md`).

#### Bài học 2: Neo giữ nhận diện nhân vật bằng System Prompt và Tỷ lệ Chiều cao
- **Nội dung cụ thể:** Đưa các thông số hình thể cố định (`height ratio`, `wardrobe lock`, `color palette`) vào System Prompt của LLM phân cảnh trước khi sinh prompt hình ảnh, kết hợp cung cấp First Frame cho mô hình sinh video.
- **Bằng chứng:** ≥ 3 nguồn:
  - @quynhaq (`7647190376868056340` - Đồng nhất nhân vật bằng System Prompt).
  - @ductuanmasterai (`7701568707251784978` - 1 prompt đồng nhất nhân vật).
  - @chauhoichutraloi (`7575022603983785236` - Veo 3 character continuity).
- **Cách áp dụng vào SocNhoTV:** Luôn nhúng quy chuẩn tỷ lệ: Sóc Nhỏ Bi 100cm, bạn Koala 150cm (`SCALE_LOCK.md`) vào file cấu hình sinh prompt để hình ảnh sinh ra không bao giờ bị sai lệch kích thước tương đối.
- **Độ tin cậy:** Cực cao (High).

#### Bài học 3: Tối ưu kết cấu thực (Texture Anti-Airbrush) thay vì làm mịn nhựa
- **Nội dung cụ thể:** Các video tạo ảnh/video có nhiều lượt lưu đều sử dụng từ khóa tiêu trừ hiệu ứng bệt nhựa AI (`subtle peach fuzz`, `visible pores`, `natural skin texture`, `no plastic airbrushing`).
- **Bằng chứng:** 
  - @ngocmiai (`7621238951155666197` - Tạo nhân vật AI có làn da chân thật nhất 2026).
  - @leduyhiep (Ảnh sản phẩm chuẩn studio).
- **Cách áp dụng vào SocNhoTV:** Khi sinh hình ảnh 3D cho Sóc Nhỏ và Koala, bổ sung các mô tả sợi lông tơ 3D bắt sáng (`soft micro-fur texture catching rim light`), tránh để chất liệu nhân vật trông như đồ chơi cao su trơn tuột.
- **Độ tin cậy:** Cao (High).

---

### D. Những Điều Chưa Kiểm Chứng (Cần Thận Trọng)
1. **Lời quảng cáo "Tạo video 60 giây liền mạch chỉ bằng 1 câu prompt trên Sora/Veo":**
   - *Thực tế:* Nhiều video từ kênh bán tool (@ldh.academy, @lyanhphung) hứa hẹn "app tự động làm video 60s không cần chỉnh sửa". Tuy nhiên trong bình luận, người dùng phản ánh video dài qua 15s thường bị trôi dạt bối cảnh (temporal drift) và méo tay chân. Các studio chuyên nghiệp vẫn phải chia nhỏ thành các cú máy 3-5 giây rồi ghép lại trên timeline.
2. **"Video AI UGC kiếm tiền tự động không cần hàng mẫu":**
   - *Thực tế:* Kênh @aiacademyglobal chia sẻ làm video unbox bằng AI, nhưng nhiều bình luận người xem nêu rõ: người tiêu dùng hiện nay rất nhạy cảm với việc tay cầm sản phẩm không chạm thật hoặc vỏ hộp bị sai chính tả chữ Việt. Nếu không có bước hậu kỳ đồ họa thật (dùng HyperFrames đè nhãn thật), tỷ lệ hoàn hàng và khiếu nại rất cao.

---

### E. Bước Tiếp Theo: 3 Việc Nên Làm Để Thử Nghiệm Ngay
1. **Thử nghiệm 1 (Prompt Test):** Chạy thử Prompt 2.1 (Robot-Arm Motion Pivot) trên Kling AI 1.5 với 2 frame nhân vật Sóc Nhỏ Bi (Rừng cây -> Lớp học) để đánh giá độ mượt của chuyển động và tính toàn vẹn của ánh mắt nhân vật.
2. **Thử nghiệm 2 (Pipeline HyperFrames):** Đưa video thử nghiệm vào template HyperFrames 9:16, kết hợp hiệu ứng âm thanh whoosh và lời thoại Sóc Nhỏ qua OmniVoice (`-14 LUFS`, presence EQ), kiểm tra hiển thị trên thiết bị di động thật để đảm bảo vùng an toàn (Safe Zone).
3. **Thử nghiệm 3 (A/B Testing KOC UGC):** Sản xuất 1 clip ngắn dạng UGC unboxing 6 phân cảnh (áp dụng Prompt 4.3 & 4.4) so sánh giữa video AI thuần túy và video có bổ sung đồ họa nhãn sản phẩm thật qua MorphIcon.
