"""Design tokens, styles, typography, and Dark/Light themes for Daqiq License Generator.

Persian RTL-first styling with WCAG AA+ high contrast and professional desktop aesthetic.
"""

from __future__ import annotations

from typing import Dict, Any
from PySide6.QtGui import QFont, QColor, QPalette


FONT_FAMILY = "Vazirmatn, IRANSans, Tahoma, Segoe UI, sans-serif"
MONO_FONT_FAMILY = "Consolas, Cascadia Code, Courier New, monospace"


LIGHT_THEME_QSS = f"""
QMainWindow, QDialog {{
    background-color: #f8fafc;
    color: #0f172a;
    font-family: {FONT_FAMILY};
    font-size: 13px;
}}

QWidget {{
    font-family: {FONT_FAMILY};
    color: #0f172a;
}}

/* Header Bar */
QFrame#HeaderBar {{
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #0f172a, stop:0.6 #1e3a8a, stop:1 #0f766e);
    border-bottom: 2px solid #0284c7;
    border-radius: 0px;
}}

/* Cards & GroupBoxes */
QGroupBox {{
    background-color: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    margin-top: 14px;
    padding-top: 18px;
    padding-bottom: 12px;
    padding-left: 14px;
    padding-right: 14px;
    font-weight: bold;
    font-size: 13px;
    color: #1e293b;
}}

QGroupBox::title {{
    subcontrol-origin: margin;
    subcontrol-position: top right;
    padding: 0 8px;
    color: #0369a1;
}}

/* Tabs */
QTabWidget::pane {{
    border: 1px solid #e2e8f0;
    background-color: #f8fafc;
    border-radius: 8px;
    top: -1px;
}}

QTabBar::tab {{
    background-color: #e2e8f0;
    color: #475569;
    padding: 10px 20px;
    border-top-left-radius: 8px;
    border-top-right-radius: 8px;
    font-weight: bold;
    margin-left: 2px;
}}

QTabBar::tab:selected {{
    background-color: #ffffff;
    color: #0369a1;
    border-bottom: 2px solid #0284c7;
}}

QTabBar::tab:hover:!selected {{
    background-color: #cbd5e1;
}}

/* Inputs */
QLineEdit, QTextEdit, QPlainTextEdit, QSpinBox, QComboBox {{
    background-color: #ffffff;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    padding: 6px 10px;
    color: #0f172a;
    selection-background-color: #0284c7;
    selection-color: #ffffff;
    font-size: 12px;
}}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus, QSpinBox:focus, QComboBox:focus {{
    border: 1px solid #0284c7;
    background-color: #f0f9ff;
}}

QLineEdit:read-only, QTextEdit:read-only {{
    background-color: #f1f5f9;
    color: #64748b;
}}

/* PushButtons */
QPushButton {{
    background-color: #0284c7;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 7px 16px;
    font-weight: bold;
    font-size: 12px;
}}

QPushButton:hover {{
    background-color: #0369a1;
}}

QPushButton:pressed {{
    background-color: #075985;
}}

QPushButton:disabled {{
    background-color: #cbd5e1;
    color: #94a3b8;
}}

/* Secondary & Action Buttons */
QPushButton#BtnSecondary {{
    background-color: #f1f5f9;
    color: #334155;
    border: 1px solid #cbd5e1;
}}

QPushButton#BtnSecondary:hover {{
    background-color: #e2e8f0;
    color: #0f172a;
}}

QPushButton#BtnSuccess {{
    background-color: #0f766e;
    color: #ffffff;
}}

QPushButton#BtnSuccess:hover {{
    background-color: #115e59;
}}

QPushButton#BtnDanger {{
    background-color: #be123c;
    color: #ffffff;
}}

QPushButton#BtnDanger:hover {{
    background-color: #9f1239;
}}

/* Tables */
QTableWidget {{
    background-color: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    gridline-color: #f1f5f9;
    selection-background-color: #e0f2fe;
    selection-color: #0369a1;
}}

QHeaderView::section {{
    background-color: #f1f5f9;
    color: #334155;
    font-weight: bold;
    padding: 8px 10px;
    border: none;
    border-bottom: 1px solid #cbd5e1;
    border-left: 1px solid #e2e8f0;
}}

/* ScrollBars */
QScrollBar:vertical {{
    background: #f1f5f9;
    width: 10px;
    margin: 0px;
    border-radius: 5px;
}}

QScrollBar::handle:vertical {{
    background: #cbd5e1;
    min-height: 20px;
    border-radius: 5px;
}}

QScrollBar::handle:vertical:hover {{
    background: #94a3b8;
}}

/* CheckBoxes & RadioButtons */
QCheckBox, QRadioButton {{
    spacing: 8px;
    font-size: 12px;
    color: #1e293b;
}}

QCheckBox::indicator, QRadioButton::indicator {{
    width: 18px;
    height: 18px;
}}

/* Badges and metric cards */
QFrame#MetricCard {{
    background-color: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    padding: 12px;
}}
"""


DARK_THEME_QSS = f"""
QMainWindow, QDialog {{
    background-color: #0f172a;
    color: #f8fafc;
    font-family: {FONT_FAMILY};
    font-size: 13px;
}}

QWidget {{
    font-family: {FONT_FAMILY};
    color: #f8fafc;
}}

/* Header Bar */
QFrame#HeaderBar {{
    background: qlineargradient(x1:0, y1:0, x2:1, y2:0, stop:0 #020617, stop:0.6 #0f172a, stop:1 #134e4a);
    border-bottom: 2px solid #0284c7;
    border-radius: 0px;
}}

/* Cards & GroupBoxes */
QGroupBox {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 10px;
    margin-top: 14px;
    padding-top: 18px;
    padding-bottom: 12px;
    padding-left: 14px;
    padding-right: 14px;
    font-weight: bold;
    font-size: 13px;
    color: #f1f5f9;
}}

QGroupBox::title {{
    subcontrol-origin: margin;
    subcontrol-position: top right;
    padding: 0 8px;
    color: #38bdf8;
}}

/* Tabs */
QTabWidget::pane {{
    border: 1px solid #334155;
    background-color: #0f172a;
    border-radius: 8px;
    top: -1px;
}}

QTabBar::tab {{
    background-color: #1e293b;
    color: #94a3b8;
    padding: 10px 20px;
    border-top-left-radius: 8px;
    border-top-right-radius: 8px;
    font-weight: bold;
    margin-left: 2px;
}}

QTabBar::tab:selected {{
    background-color: #334155;
    color: #38bdf8;
    border-bottom: 2px solid #38bdf8;
}}

QTabBar::tab:hover:!selected {{
    background-color: #243247;
}}

/* Inputs */
QLineEdit, QTextEdit, QPlainTextEdit, QSpinBox, QComboBox {{
    background-color: #1e293b;
    border: 1px solid #475569;
    border-radius: 6px;
    padding: 6px 10px;
    color: #f8fafc;
    selection-background-color: #0284c7;
    selection-color: #ffffff;
    font-size: 12px;
}}

QLineEdit:focus, QTextEdit:focus, QPlainTextEdit:focus, QSpinBox:focus, QComboBox:focus {{
    border: 1px solid #38bdf8;
    background-color: #0f172a;
}}

QLineEdit:read-only, QTextEdit:read-only {{
    background-color: #0b1120;
    color: #94a3b8;
}}

/* PushButtons */
QPushButton {{
    background-color: #0284c7;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 7px 16px;
    font-weight: bold;
    font-size: 12px;
}}

QPushButton:hover {{
    background-color: #0369a1;
}}

QPushButton:pressed {{
    background-color: #075985;
}}

QPushButton:disabled {{
    background-color: #334155;
    color: #64748b;
}}

/* Secondary & Action Buttons */
QPushButton#BtnSecondary {{
    background-color: #334155;
    color: #f1f5f9;
    border: 1px solid #475569;
}}

QPushButton#BtnSecondary:hover {{
    background-color: #475569;
}}

QPushButton#BtnSuccess {{
    background-color: #0f766e;
    color: #ffffff;
}}

QPushButton#BtnSuccess:hover {{
    background-color: #115e59;
}}

QPushButton#BtnDanger {{
    background-color: #be123c;
    color: #ffffff;
}}

QPushButton#BtnDanger:hover {{
    background-color: #9f1239;
}}

/* Tables */
QTableWidget {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    gridline-color: #334155;
    selection-background-color: #0369a1;
    selection-color: #ffffff;
}}

QHeaderView::section {{
    background-color: #0f172a;
    color: #cbd5e1;
    font-weight: bold;
    padding: 8px 10px;
    border: none;
    border-bottom: 1px solid #334155;
    border-left: 1px solid #334155;
}}

/* ScrollBars */
QScrollBar:vertical {{
    background: #0f172a;
    width: 10px;
    margin: 0px;
    border-radius: 5px;
}}

QScrollBar::handle:vertical {{
    background: #334155;
    min-height: 20px;
    border-radius: 5px;
}}

QScrollBar::handle:vertical:hover {{
    background: #475569;
}}

/* CheckBoxes & RadioButtons */
QCheckBox, QRadioButton {{
    spacing: 8px;
    font-size: 12px;
    color: #f1f5f9;
}}

QCheckBox::indicator, QRadioButton::indicator {{
    width: 18px;
    height: 18px;
}}

/* Badges and metric cards */
QFrame#MetricCard {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 10px;
    padding: 12px;
}}
"""


def get_theme_stylesheet(is_dark: bool = False) -> str:
    """Return complete QSS stylesheet for the selected theme."""
    return DARK_THEME_QSS if is_dark else LIGHT_THEME_QSS
