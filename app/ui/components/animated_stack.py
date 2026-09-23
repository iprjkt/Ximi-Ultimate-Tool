"""
Animated Stacked Widget for Ximi Ultimate Tool.
Provides fluid slide & fade page transitions with smooth easing curves (HyperOS / iOS style).
"""

from PyQt6.QtCore import (
    QEasingCurve,
    QParallelAnimationGroup,
    QPoint,
    QPropertyAnimation,
    Qt
)
from PyQt6.QtWidgets import (
    QGraphicsOpacityEffect,
    QStackedWidget,
    QWidget
)

class AnimatedStackedWidget(QStackedWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.animation_duration = 260
        self.animating = False

    def slide_to_index(self, next_index: int):
        current_index = self.currentIndex()
        if current_index == next_index or self.animating:
            return

        if not (0 <= next_index < self.count()):
            return

        current_widget = self.widget(current_index)
        next_widget = self.widget(next_index)

        if not current_widget or not next_widget:
            self.setCurrentIndex(next_index)
            return

        self.animating = True

        width = self.frameGeometry().width()
        height = self.frameGeometry().height()

        # Determine slide direction
        slide_left = (next_index > current_index)
        offset_x = int(width * 0.35)  # 35% slide distance feels fast, fluid, and natural

        start_next_x = offset_x if slide_left else -offset_x
        end_curr_x = -offset_x if slide_left else offset_x

        # Setup next widget
        next_widget.setGeometry(0, 0, width, height)
        next_widget.show()
        next_widget.raise_()

        # Opacity effects
        effect_curr = QGraphicsOpacityEffect(current_widget)
        current_widget.setGraphicsEffect(effect_curr)
        effect_next = QGraphicsOpacityEffect(next_widget)
        next_widget.setGraphicsEffect(effect_next)

        # 1. Slide Animation Current Widget
        anim_curr_pos = QPropertyAnimation(current_widget, b"pos")
        anim_curr_pos.setDuration(self.animation_duration)
        anim_curr_pos.setEasingCurve(QEasingCurve.Type.OutCubic)
        anim_curr_pos.setStartValue(QPoint(0, 0))
        anim_curr_pos.setEndValue(QPoint(end_curr_x, 0))

        # 2. Fade Out Current Widget
        anim_curr_fade = QPropertyAnimation(effect_curr, b"opacity")
        anim_curr_fade.setDuration(int(self.animation_duration * 0.75))
        anim_curr_fade.setStartValue(1.0)
        anim_curr_fade.setEndValue(0.0)

        # 3. Slide Animation Next Widget
        anim_next_pos = QPropertyAnimation(next_widget, b"pos")
        anim_next_pos.setDuration(self.animation_duration)
        anim_next_pos.setEasingCurve(QEasingCurve.Type.OutCubic)
        anim_next_pos.setStartValue(QPoint(start_next_x, 0))
        anim_next_pos.setEndValue(QPoint(0, 0))

        # 4. Fade In Next Widget
        anim_next_fade = QPropertyAnimation(effect_next, b"opacity")
        anim_next_fade.setDuration(self.animation_duration)
        anim_next_fade.setEasingCurve(QEasingCurve.Type.OutCubic)
        anim_next_fade.setStartValue(0.0)
        anim_next_fade.setEndValue(1.0)

        # Group Animations
        self.group = QParallelAnimationGroup(self)
        self.group.addAnimation(anim_curr_pos)
        self.group.addAnimation(anim_curr_fade)
        self.group.addAnimation(anim_next_pos)
        self.group.addAnimation(anim_next_fade)

        def on_finished():
            self.setCurrentIndex(next_index)
            current_widget.move(0, 0)
            next_widget.move(0, 0)
            current_widget.setGraphicsEffect(None)
            next_widget.setGraphicsEffect(None)
            self.animating = False

        self.group.finished.connect(on_finished)
        self.group.start()
