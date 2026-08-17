# ai-limits

<p align="center">| <a href="DE.md">DE</a> | <a href="../../README.md">EN</a> | <a href="ES.md">ES</a> | <a href="FR.md">FR</a> | <a href="PT.md">PT</a> | <a href="RU.md">RU</a> | <a href="ZH.md">中文</a> | عربي |</p>

<p align="center" dir="rtl">
   تطبيق محلي لمراقبة حدود واستهلاك اشتراكات الذكاء الاصطناعي في Codex وClaude وCursor.
</p>

<p align="center">
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-macos-arm64.dmg"><img src="https://shieldcn.dev/badge/macOS-v0.4.0-grey.svg?logo=apple" alt="تنزيل لنظام macOS"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-windows-setup.exe"><img src="https://shieldcn.dev/badge/Windows-v0.4.0-blue.svg?logo=ri:FaWindows" alt="تنزيل لنظام Windows"></a>
  <a href="https://github.com/md2it/ai-limits/releases/download/v0.4.0/AI-Limits-v0.4.0-linux.AppImage"><img src="https://shieldcn.dev/badge/Linux-v0.4.0-yellow.svg?logo=linux" alt="تنزيل لنظام Linux"></a>
</p>

<p align="center"><a href="https://github.com/md2it/ai-limits/releases/tag/v0.4.0">كل التنزيلات</a></p>

---

![ai-limits macOS](screenshots/macos.png)

<p align="center">
  <img src="screenshots/windows.png" alt="ai-limits على Windows" width="24%">
  <img src="screenshots/linux.png" alt="ai-limits على Linux" width="24%">
  <img src="screenshots/macos-light-settings.png" alt="إعدادات ai-limits" width="24%">
  <img src="screenshots/macos-help.png" alt="مساعدة ai-limits" width="24%">
</p>

<p align="center">
  <img src="screenshots/macos-popover-dark.png" alt="نافذة ai-limits المنبثقة على macOS بالمظهر الداكن" width="24%">
  <img src="screenshots/macos-popover-light.png" alt="نافذة ai-limits المنبثقة على macOS بالمظهر الفاتح" width="24%">
  <img src="screenshots/macos-notification-center-dark.png" alt="إشعارات ai-limits في مركز إشعارات macOS" width="24%">
  <img src="screenshots/macos-notification-center-light.png" alt="إشعارات ai-limits الفاتحة في مركز إشعارات macOS" width="24%">
</p>

<div dir="rtl">

## المزايا

- يعمل دون اشتراك في واجهة برمجة التطبيقات (API)،
- لا يحتاج إلى حساب AI Limits منفصل: يستخدم تفويض المزوّد الحالي،
- جميع المزوّدين في مكان واحد،
- مجاني بالكامل،
- يحافظ على الخصوصية. بلا خدمات طرف ثالث أو وكلاء أو تسجيلات،
- تطبيق سطح مكتب خفيف لنظام Mac وWindows وLinux،
- إشعارات بالحدود،
- مفتوح المصدر.

## الميزات

- يعرض الحدود وتاريخ ووقت إعادة التعيين والرموز المتاحة وإعادة التعيين اليدوية المتاحة،
- يعمل مع Codex وClaude وCursor،
- يستخرج البيانات من الملفات المحلية وواجهات سطر أوامر المزوّدين وواجهات برمجة التطبيقات،
- منطق احتياطي: إذا تعذّر الوصول إلى مصدر، يتحقق من مصدر آخر،
- تطبيق سطح مكتب خفيف (macOS وWindows وLinux)،
- واجهة سطر أوامر بعدة صيغ إخراج،
- إشعارات النظام عند بلوغ عتبات الحدود،
- تحديث يدوي لجميع البيانات وتردد موحد للتحديث التلقائي.

## البدائل

| | **ai-limits** | [CodexBar](https://github.com/steipete/CodexBar) | [caut](https://github.com/Dicklesworthstone/coding_agent_usage_tracker) |
| --- | :---: | :---: | :---: |
| تطبيق سطح مكتب وواجهة سطر أوامر | ✅ | ✅ | ❌ |
| Codex وClaude وCursor | ✅ | ✅ | ✅ |
| macOS وWindows وLinux | ✅ | ❌ | ❌ |
| بلا خدمة وسيطة | ✅ | ✅ | ✅ |

المقارنة الكاملة لـ 18 بديلاً و16 معياراً متوفرة في [كتالوج البدائل](../product/analogues.tsv).

## دعم المنصات والقيود

- macOS: إصدار مدعوم؛ التطبيق موقّع ومُوثَّق ومرفق بتذكرة؛ والإشعارات تعمل،
- Windows وLinux: تتوفر إصدارات ما قبل النشر غير الموقّعة؛ ويتطوّر الدعم استناداً إلى ملاحظات المستخدمين،
- إشعارات تطبيق سطح المكتب متاحة حالياً على macOS فقط،
- قد لا تعمل بعض المصادر المحلية لـ Codex وClaude على Windows وLinux بعد (المصادر عبر واجهة سطر الأوامر تعمل في كل مكان)،
- تتطلب مصادر CLI لـ Codex وClaude تفويض المزوّد المعني؛ ويتطلب Cursor رمزاً صالحاً من Cursor Agent مفوَّض.

## الترخيص

[رخصة MIT](../../LICENSE)

</div>
