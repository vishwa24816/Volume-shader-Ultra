use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Event, HtmlButtonElement, HtmlCanvasElement, HtmlElement, HtmlTextAreaElement, MouseEvent,
    TouchEvent, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlUniformLocation,
    WheelEvent,
};

const VERT_SHADER_SOURCE: &str = "#version 100 \n\
precision highp float;\n\
attribute vec4 position;\
varying vec3 dir, localdir;\
uniform vec3 right, forward, up, origin;\
uniform float x,y;\
void main() {\
   gl_Position = position; \
   dir = forward + right * position.x*x + up * position.y*y;\
   localdir.x = position.x*x;\
   localdir.y = position.y*y;\
   localdir.z = -1.0;\
} ";

const FRAG_SHADER_SOURCE: &str = "#version 100 \n\
#define PI 3.14159265358979324\n\
#define M_L 0.3819660113\n\
#define M_R 0.6180339887\n\
#define MAXR 8\n\
#define SOLVER 8\n\
precision highp float;\n\
float kernal(vec3 ver)\n;\
uniform vec3 right, forward, up, origin;\
varying vec3 dir, localdir;\
uniform float len;\
vec3 ver;\
int sign;\
float v, v1, v2;\
float r1, r2, r3, r4, m1, m2, m3, m4;\
vec3 n, reflect;\
const float step = 0.002;\
vec3 color;\
void main() {\
   color.r=0.0;\
   color.g=0.0;\
   color.b=0.0;\
   sign=0;\
   v1 = kernal(origin + dir * (step*len));\
   v2 = kernal(origin);\
   for (int k = 2; k < 1002; k++) {\
      ver = origin + dir * (step*len*float(k));\
      v = kernal(ver);\
      if (v > 0.0 && v1 < 0.0) {\
         r1 = step * len*float(k - 1);\
         r2 = step * len*float(k);\
         m1 = kernal(origin + dir * r1);\
         m2 = kernal(origin + dir * r2);\
         for (int l = 0; l < SOLVER; l++) {\
            r3 = r1 * 0.5 + r2 * 0.5;\
            m3 = kernal(origin + dir * r3);\
            if (m3 > 0.0) {\
               r2 = r3;\
               m2 = m3;\
            }\
            else {\
               r1 = r3;\
               m1 = m3;\
            }\
         }\
         if (r3 < 2.0 * len) {\
               sign=1;\
            break;\
         }\
      }\
      if (v < v1&&v1>v2&&v1 < 0.0 && (v1*2.0 > v || v1 * 2.0 > v2)) {\
         r1 = step * len*float(k - 2);\
         r2 = step * len*(float(k) - 2.0 + 2.0*M_L);\
         r3 = step * len*(float(k) - 2.0 + 2.0*M_R);\
         r4 = step * len*float(k);\
         m2 = kernal(origin + dir * r2);\
         m3 = kernal(origin + dir * r3);\
         for (int l = 0; l < MAXR; l++) {\
            if (m2 > m3) {\
               r4 = r3;\
               r3 = r2;\
               r2 = r4 * M_L + r1 * M_R;\
               m3 = m2;\
               m2 = kernal(origin + dir * r2);\
            }\
            else {\
               r1 = r2;\
               r2 = r3;\
               r3 = r4 * M_R + r1 * M_L;\
               m2 = m3;\
               m3 = kernal(origin + dir * r3);\
            }\
         }\
         if (m2 > 0.0) {\
            r1 = step * len*float(k - 2);\
            r2 = r2;\
            m1 = kernal(origin + dir * r1);\
            m2 = kernal(origin + dir * r2);\
            for (int l = 0; l < SOLVER; l++) {\
               r3 = r1 * 0.5 + r2 * 0.5;\
               m3 = kernal(origin + dir * r3);\
               if (m3 > 0.0) {\
                  r2 = r3;\
                  m2 = m3;\
               }\
               else {\
                  r1 = r3;\
                  m1 = m3;\
               }\
            }\
            if (r3 < 2.0 * len&&r3> step*len) {\
                   sign=1;\
               break;\
            }\
         }\
         else if (m3 > 0.0) {\
            r1 = step * len*float(k - 2);\
            r2 = r3;\
            m1 = kernal(origin + dir * r1);\
            m2 = kernal(origin + dir * r2);\
            for (int l = 0; l < SOLVER; l++) {\
               r3 = r1 * 0.5 + r2 * 0.5;\
               m3 = kernal(origin + dir * r3);\
               if (m3 > 0.0) {\
                  r2 = r3;\
                  m2 = m3;\
               }\
               else {\
                  r1 = r3;\
                  m1 = m3;\
               }\
            }\
            if (r3 < 2.0 * len&&r3> step*len) {\
                   sign=1;\
               break;\
            }\
         }\
      }\
      v2 = v1;\
      v1 = v;\
   }\
   if (sign==1) {\
      ver = origin + dir*r3 ;\
       r1=ver.x*ver.x+ver.y*ver.y+ver.z*ver.z;\
      n.x = kernal(ver - right * (r3*0.00025)) - kernal(ver + right * (r3*0.00025));\
      n.y = kernal(ver - up * (r3*0.00025)) - kernal(ver + up * (r3*0.00025));\
      n.z = kernal(ver + forward * (r3*0.00025)) - kernal(ver - forward * (r3*0.00025));\
      r3 = n.x*n.x+n.y*n.y+n.z*n.z;\
      n = n * (1.0 / sqrt(r3));\
      ver = localdir;\
      r3 = ver.x*ver.x+ver.y*ver.y+ver.z*ver.z;\
      ver = ver * (1.0 / sqrt(r3));\
      reflect = n * (-2.0*dot(ver, n)) + ver;\
      r3 = reflect.x*0.276+reflect.y*0.920+reflect.z*0.276;\
      r4 = n.x*0.276+n.y*0.920+n.z*0.276;\
      r3 = max(0.0,r3);\
      r3 = r3 * r3*r3*r3;\
      r3 = r3 * 0.45 + r4 * 0.25 + 0.3;\
      n.x = sin(r1*10.0)*0.5+0.5;\
      n.y = sin(r1*10.0+2.05)*0.5+0.5;\
      n.z = sin(r1*10.0-2.05)*0.5+0.5;\
      color = n*r3;\
   }\
   gl_FragColor = vec4(color.x, color.y, color.z, 1.0);\
}";

const DEFAULT_KERNEL: &str = "float kernal(vec3 ver){\n\
   vec3 a;\n\
float b,c,d,e;\n\
   a=ver;\n\
   for(int i=0;i<5;i++){\n\
       b=length(a);\n\
       c=atan(a.y,a.x)*8.0;\n\
       e=1.0/b;\n\
       d=acos(a.z/b)*8.0;\n\
       b=pow(b,8.0);\n\
       a=vec3(b*sin(d)*cos(c),b*sin(d)*sin(c),b*cos(d))+ver;\n\
       if(b>6.0){\n\
           break;\n\
       }\n\
   }\n\
   return 4.0-a.x*a.x-a.y*a.y-a.z*a.z;\n\
}";

const POSITIONS: [f32; 18] = [
    -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0,
    1.0, 0.0,
];

struct App {
    cx: f64,
    cy: f64,
    len: f64,
    ang1: f64,
    ang2: f64,
    cenx: f64,
    ceny: f64,
    cenz: f64,
    mx: f64,
    my: f64,
    mx1: f64,
    my1: f64,
    lasttimen: i32,
    ml: i32,
    mr: i32,
    mm: i32,
    kernel: String,
    gl: WebGlRenderingContext,
    #[allow(dead_code)]
    vert_shader: WebGlShader,
    frag_shader: WebGlShader,
    shader_program: WebGlProgram,
    #[allow(dead_code)]
    buffer: WebGlBuffer,
    glposition: u32,
    glright: Option<WebGlUniformLocation>,
    glforward: Option<WebGlUniformLocation>,
    glup: Option<WebGlUniformLocation>,
    glorigin: Option<WebGlUniformLocation>,
    glx: Option<WebGlUniformLocation>,
    gly: Option<WebGlUniformLocation>,
    gllen: Option<WebGlUniformLocation>,
    main_el: HtmlElement,
    btn: HtmlButtonElement,
    config: HtmlElement,
    apply: HtmlButtonElement,
    cancle: HtmlButtonElement,
    kernel_ta: HtmlTextAreaElement,
    fps_el: HtmlElement,
    fps_frames: u32,
    fps_last: f64,
    fps_value: f64,
}

impl App {
    fn compile_shader(
        gl: &WebGlRenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, String> {
        let shader = gl
            .create_shader(shader_type)
            .ok_or_else(|| "create_shader failed".to_string())?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        Ok(shader)
    }

    fn link_program(
        gl: &WebGlRenderingContext,
        vert: &WebGlShader,
        frag: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = gl
            .create_program()
            .ok_or_else(|| "create_program failed".to_string())?;
        gl.attach_shader(&program, vert);
        gl.attach_shader(&program, frag);
        gl.link_program(&program);
        Ok(program)
    }

    fn refresh_locations(&mut self) {
        let gl = &self.gl;
        let p = &self.shader_program;
        self.glposition = gl
            .get_attrib_location(p, "position")
            .max(0) as u32;
        self.glright = gl.get_uniform_location(p, "right");
        self.glforward = gl.get_uniform_location(p, "forward");
        self.glup = gl.get_uniform_location(p, "up");
        self.glorigin = gl.get_uniform_location(p, "origin");
        self.glx = gl.get_uniform_location(p, "x");
        self.gly = gl.get_uniform_location(p, "y");
        self.gllen = gl.get_uniform_location(p, "len");
    }

    fn recompile_fragment(&mut self) -> Result<(), String> {
        let gl = &self.gl;
        gl.shader_source(&self.frag_shader, &format!("{}{}", FRAG_SHADER_SOURCE, self.kernel));
        gl.compile_shader(&self.frag_shader);
        gl.link_program(&self.shader_program);
        gl.use_program(Some(&self.shader_program));
        if !gl
            .get_program_parameter(&self.shader_program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let infof = gl
                .get_shader_info_log(&self.frag_shader)
                .unwrap_or_default();
            let info = gl
                .get_program_info_log(&self.shader_program)
                .unwrap_or_default();
            return Err(format!("{}{}", infof, info));
        }
        self.refresh_locations();
        Ok(())
    }

    fn draw(&self) {
        let gl = &self.gl;
        let sum = self.cx + self.cy;
        gl.uniform1f(self.glx.as_ref(), (self.cx * 2.0 / sum) as f32);
        gl.uniform1f(self.gly.as_ref(), (self.cy * 2.0 / sum) as f32);
        gl.uniform1f(self.gllen.as_ref(), self.len as f32);
        gl.uniform3f(
            self.glorigin.as_ref(),
            (self.len * self.ang1.cos() * self.ang2.cos() + self.cenx) as f32,
            (self.len * self.ang2.sin() + self.ceny) as f32,
            (self.len * self.ang1.sin() * self.ang2.cos() + self.cenz) as f32,
        );
        gl.uniform3f(
            self.glright.as_ref(),
            self.ang1.sin() as f32,
            0.0,
            -self.ang1.cos() as f32,
        );
        gl.uniform3f(
            self.glup.as_ref(),
            (-self.ang2.sin() * self.ang1.cos()) as f32,
            self.ang2.cos() as f32,
            (-self.ang2.sin() * self.ang1.sin()) as f32,
        );
        gl.uniform3f(
            self.glforward.as_ref(),
            (-self.ang1.cos() * self.ang2.cos()) as f32,
            -self.ang2.sin() as f32,
            (-self.ang1.sin() * self.ang2.cos()) as f32,
        );
        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
        gl.finish();
    }

    fn fit_canvas(&mut self) {
        let window = web_sys::window().expect("no window");
        let mut cx = window
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let mut cy = window
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if cx > cy {
            cx = cy;
        } else {
            cy = cx;
        }
        self.cx = cx;
        self.cy = cy;
        let _ = self.main_el.style().set_property("width", "1024px");
        let _ = self.main_el.style().set_property("height", "1024px");
        let _ = self
            .main_el
            .style()
            .set_property("transform", &format!("scale({},{})", cx / 1024.0, cy / 1024.0));
    }

    fn update_fps(&mut self, now: f64) {
        self.fps_frames += 1;
        let elapsed = now - self.fps_last;
        if elapsed >= 0.5 {
            self.fps_value = (self.fps_frames as f64) * 1000.0 / elapsed;
            self.fps_frames = 0;
            self.fps_last = now;
            let text = format!("FPS: {:.0}", self.fps_value);
            self.fps_el.set_inner_text(&text);
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    let alert_msg = "trigonometric and inverse trigonometric functions test.\ncreated by cznull@bilibili";
    window.alert_with_message(alert_msg)?;

    let canvas = document
        .get_element_by_id("c1")
        .expect("missing #c1")
        .dyn_into::<HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .expect("webgl not supported")
        .dyn_into()?;

    let vert_shader = App::compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, VERT_SHADER_SOURCE)
        .map_err(|e| JsValue::from_str(&e))?;
    let frag_shader = App::compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        &format!("{}{}", FRAG_SHADER_SOURCE, DEFAULT_KERNEL),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    let shader_program =
        App::link_program(&gl, &vert_shader, &frag_shader).map_err(|e| JsValue::from_str(&e))?;
    gl.use_program(Some(&shader_program));
    if !gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let infov = gl.get_shader_info_log(&vert_shader).unwrap_or_default();
        let infof = gl.get_shader_info_log(&frag_shader).unwrap_or_default();
        let info = gl.get_program_info_log(&shader_program).unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "Could not compile WebGL program. \n\n{}{}{}",
            infov, infof, info
        )));
    }

    let buffer = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("create_buffer failed"))?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    let positions = js_sys::Float32Array::from(POSITIONS.as_ref());
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &positions,
        WebGlRenderingContext::STATIC_DRAW,
    );

    let app = Rc::new(RefCell::new(App {
        cx: 0.0,
        cy: 0.0,
        len: 1.6,
        ang1: 2.8,
        ang2: 0.4,
        cenx: 0.0,
        ceny: 0.0,
        cenz: 0.0,
        mx: 0.0,
        my: 0.0,
        mx1: 0.0,
        my1: 0.0,
        lasttimen: 0,
        ml: 0,
        mr: 0,
        mm: 0,
        kernel: DEFAULT_KERNEL.to_string(),
        gl,
        vert_shader,
        frag_shader,
        shader_program,
        buffer,
        glposition: 0,
        glright: None,
        glforward: None,
        glup: None,
        glorigin: None,
        glx: None,
        gly: None,
        gllen: None,
        main_el: document
            .get_element_by_id("main")
            .expect("missing #main")
            .dyn_into::<HtmlElement>()?,
        btn: document
            .get_element_by_id("btn")
            .expect("missing #btn")
            .dyn_into::<HtmlButtonElement>()?,
        config: document
            .get_element_by_id("config")
            .expect("missing #config")
            .dyn_into::<HtmlElement>()?,
        apply: document
            .get_element_by_id("apply")
            .expect("missing #apply")
            .dyn_into::<HtmlButtonElement>()?,
        cancle: document
            .get_element_by_id("cancle")
            .expect("missing #cancle")
            .dyn_into::<HtmlButtonElement>()?,
        kernel_ta: document
            .get_element_by_id("kernel")
            .expect("missing #kernel")
            .dyn_into::<HtmlTextAreaElement>()?,
        fps_el: document
            .get_element_by_id("fps")
            .expect("missing #fps")
            .dyn_into::<HtmlElement>()?,
        fps_frames: 0,
        fps_last: 0.0,
        fps_value: 0.0,
    }));

    {
        let mut app = app.borrow_mut();
        app.glposition = app
            .gl
            .get_attrib_location(&app.shader_program, "position")
            .max(0) as u32;
        app.glright = app.gl.get_uniform_location(&app.shader_program, "right");
        app.glforward = app.gl.get_uniform_location(&app.shader_program, "forward");
        app.glup = app.gl.get_uniform_location(&app.shader_program, "up");
        app.glorigin = app.gl.get_uniform_location(&app.shader_program, "origin");
        app.glx = app.gl.get_uniform_location(&app.shader_program, "x");
        app.gly = app.gl.get_uniform_location(&app.shader_program, "y");
        app.gllen = app.gl.get_uniform_location(&app.shader_program, "len");
        app.gl.vertex_attrib_pointer_with_i32(app.glposition, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        app.gl.enable_vertex_attrib_array(app.glposition);
        app.gl.viewport(0, 0, 1024, 1024);
    }

    app.borrow_mut().fit_canvas();
    app.borrow().draw();

    // CONFIG button toggle
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
            let a = app_c.borrow_mut();
            let state = a.btn.inner_text() == "CONFIG";
            a.btn.set_inner_text(if state { "HIDE" } else { "CONFIG" });
            let _ = a
                .config
                .style()
                .set_property("display", if state { "inline" } else { "none" });
        }) as Box<dyn FnMut(MouseEvent)>);
        app.borrow().btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // APPLY button
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
            let mut a = app_c.borrow_mut();
            a.kernel = a.kernel_ta.value();
            if let Err(e) = a.recompile_fragment() {
                let _ = web_sys::window().unwrap().alert_with_message(&e);
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        app.borrow().apply.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // CANCLE button
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |_ev: MouseEvent| {
            let a = app_c.borrow();
            a.kernel_ta.set_value(&a.kernel);
        }) as Box<dyn FnMut(MouseEvent)>);
        app.borrow().cancle.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // mouse events
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
            let mut a = app_c.borrow_mut();
            if ev.button() == 0 {
                a.ml = 1;
                a.mm = 0;
            }
            if ev.button() == 2 {
                a.mr = 1;
                a.mm = 0;
            }
            a.mx = ev.client_x() as f64;
            a.my = ev.client_y() as f64;
        }) as Box<dyn FnMut(MouseEvent)>);
        document.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
            let mut a = app_c.borrow_mut();
            if ev.button() == 0 {
                a.ml = 0;
            }
            if ev.button() == 2 {
                a.mr = 0;
            }
        }) as Box<dyn FnMut(MouseEvent)>);
        document.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: MouseEvent| {
            let mut a = app_c.borrow_mut();
            let dx = ev.client_x() as f64 - a.mx;
            let dy = ev.client_y() as f64 - a.my;
            if a.ml == 1 {
                a.ang1 += dx * 0.002;
                a.ang2 += dy * 0.002;
                if ev.client_x() as f64 != a.mx || ev.client_y() as f64 != a.my {
                    a.mm = 1;
                }
            }
            if a.mr == 1 {
                let l = a.len * 4.0 / (a.cx + a.cy);
                a.cenx += l * (-dx * a.ang1.sin() - dy * a.ang2.sin() * a.ang1.cos());
                a.ceny += l * (dy * a.ang2.cos());
                a.cenz += l * (dx * a.ang1.cos() - dy * a.ang2.sin() * a.ang1.sin());
                if ev.client_x() as f64 != a.mx || ev.client_y() as f64 != a.my {
                    a.mm = 1;
                }
            }
            a.mx = ev.client_x() as f64;
            a.my = ev.client_y() as f64;
        }) as Box<dyn FnMut(MouseEvent)>);
        document.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: WheelEvent| {
            ev.prevent_default();
            let mut a = app_c.borrow_mut();
            let delta = js_sys::Reflect::get(&ev, &JsValue::from_str("wheelDelta"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            a.len *= (-0.001 * delta).exp();
        }) as Box<dyn FnMut(WheelEvent)>);
        document.add_event_listener_with_callback("mousewheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: Event| {
            let a = app_c.borrow();
            if a.mm == 1 {
                ev.prevent_default();
            }
        }) as Box<dyn FnMut(Event)>);
        document.set_oncontextmenu(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // touch events
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: TouchEvent| {
            let n = ev.touches().length();
            let mut a = app_c.borrow_mut();
            if n == 1 {
                let t = ev.touches().get(0).expect("touch");
                a.mx = t.client_x() as f64;
                a.my = t.client_y() as f64;
            } else if n == 2 {
                let t0 = ev.touches().get(0).expect("touch");
                a.mx = t0.client_x() as f64;
                a.my = t0.client_y() as f64;
                let t1 = ev.touches().get(1).expect("touch");
                a.mx1 = t1.client_x() as f64;
                a.my1 = t1.client_y() as f64;
            }
            a.lasttimen = n as i32;
        }) as Box<dyn FnMut(TouchEvent)>);
        document.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: TouchEvent| {
            let n = ev.touches().length();
            let mut a = app_c.borrow_mut();
            if n == 1 {
                let t = ev.touches().get(0).expect("touch");
                a.mx = t.client_x() as f64;
                a.my = t.client_y() as f64;
            } else if n == 2 {
                let t0 = ev.touches().get(0).expect("touch");
                a.mx = t0.client_x() as f64;
                a.my = t0.client_y() as f64;
                let t1 = ev.touches().get(1).expect("touch");
                a.mx1 = t1.client_x() as f64;
                a.my1 = t1.client_y() as f64;
            }
            a.lasttimen = n as i32;
        }) as Box<dyn FnMut(TouchEvent)>);
        document.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |ev: TouchEvent| {
            ev.prevent_default();
            let n = ev.touches().length();
            let mut a = app_c.borrow_mut();
            if n == 1 && a.lasttimen == 1 {
                let t = ev.touches().get(0).expect("touch");
                a.ang1 += (t.client_x() as f64 - a.mx) * 0.002;
                a.ang2 += (t.client_y() as f64 - a.my) * 0.002;
                a.mx = t.client_x() as f64;
                a.my = t.client_y() as f64;
            } else if n == 2 {
                let t0 = ev.touches().get(0).expect("touch");
                let t1 = ev.touches().get(1).expect("touch");
                let l = a.len * 2.0 / (a.cx + a.cy);
                let sx = (t0.client_x() + t1.client_x()) as f64 - a.mx - a.mx1;
                let sy = (t0.client_y() + t1.client_y()) as f64 - a.my - a.my1;
                a.cenx += l * (-sx * a.ang1.sin() - sy * a.ang2.sin() * a.ang1.cos());
                a.ceny += l * (sy * a.ang2.cos());
                a.cenz += l * (sx * a.ang1.cos() - sy * a.ang2.sin() * a.ang1.sin());
                let l1 = ((a.mx - a.mx1) * (a.mx - a.mx1) + (a.my - a.my1) * (a.my - a.my1) + 1.0).sqrt();
                a.mx = t0.client_x() as f64;
                a.my = t0.client_y() as f64;
                a.mx1 = t1.client_x() as f64;
                a.my1 = t1.client_y() as f64;
                let l = ((a.mx - a.mx1) * (a.mx - a.mx1) + (a.my - a.my1) * (a.my - a.my1) + 1.0).sqrt();
                a.len *= l1 / l;
            }
            a.lasttimen = n as i32;
        }) as Box<dyn FnMut(TouchEvent)>);
        document.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // resize
    {
        let app_c = app.clone();
        let closure = Closure::wrap(Box::new(move |_ev: Event| {
            app_c.borrow_mut().fit_canvas();
        }) as Box<dyn FnMut(Event)>);
        window.set_onresize(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // kernel textarea initial value
    app.borrow().kernel_ta.set_value(DEFAULT_KERNEL);

    // render loop
    {
        let app_c = app.clone();
        let window_c = window.clone();
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let mut a = app_c.borrow_mut();
            a.ang1 += 0.01;
            a.draw();
            a.update_fps(js_sys::Date::now());
            let _ = window_c.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }) as Box<dyn FnMut()>));
        window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    }

    // keep spawn_local import used (future-proofing for async init if needed)
    spawn_local(async {});

    Ok(())
}
